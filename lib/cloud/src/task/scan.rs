use log::{error, info, warn};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::sync::Arc;
use std::time::Duration;
use chrono::{Local, TimeDelta};
use tokio::sync::Semaphore;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::{CloudFileBlock, EventMessage, EventType, FileStatus, MetaStatus, TempCloudFile};
use service::meta::{CloudMetaManager, FileManager};
use service::CONTEXT;
use storage::storage_facade;

/// 上传连续失败达到该次数后进入死信（FileStatus::Dead）
const UPLOAD_MAX_ATTEMPTS: i32 = 10;
/// 退避基数（秒）：第 n 次失败后等待 base * 2^(n-1)，封顶 1 小时
const UPLOAD_BACKOFF_BASE_SECONDS: i64 = 60;

/// 指数退避 + 抖动：base * 2^(attempt-1)，封顶 1 小时，加 0..base/2 随机抖动
fn backoff_after(attempt: i32) -> TimeDelta {
    let exp = (attempt - 1).clamp(0, 6) as i64;
    let base = dotenvy::var("UPLOAD_BACKOFF_BASE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(UPLOAD_BACKOFF_BASE_SECONDS)
        .max(1);
    let mut seconds = base.saturating_mul(1 << exp);
    seconds = seconds.min(3600);
    let jitter = (seconds / 2).max(1);
    TimeDelta::seconds(seconds) + TimeDelta::seconds(rand::random::<i64>() % jitter)
}

pub(crate) async fn scan(semaphore: Arc<Semaphore>) -> ResponseResult<()> {
    let now = Local::now();
    let cloud_file_block_result = CONTEXT.cloud_file_block_manager.select_to_upload(now).await;
    if let Err(e) = cloud_file_block_result {
        error!("select_to_upload error {:?}", e);
        return Ok(());
    }
    let cloud_file_blocks = cloud_file_block_result?;
    for file_block in cloud_file_blocks {
        let semaphore = semaphore.clone();
        tokio::spawn(async move {
            if let Err(e) = process_block(file_block, semaphore.clone()).await {
                error!("process_block failed: {}", e);
            }
        });
    }
    Ok(())
}

async fn process_block(mut file_block: CloudFileBlock, semaphore: Arc<Semaphore>) -> ResponseResult<()> {
    let semaphore_permit_result = semaphore.try_acquire();
    if let Err(e) = semaphore_permit_result {
        error!("{}",e);
        return Ok(());
    }
    info!("available_permits:{}",semaphore.available_permits());
    let origin_status = file_block.status.try_into()?;
    file_block.update_time = Local::now();
    file_block.status = FileStatus::Uploading.into();
    let count = CONTEXT.cloud_file_block_manager.update_by_status(&file_block, origin_status)
        .await?;
    if count == 0 {
        info!("block {} unable lock by db", file_block.id.unwrap());
        return Ok(());
    }
    // 通过信号量控制并发
    info!("upload block {} to cloud", file_block.id.unwrap());
    let result = do_execute_one_block(&mut file_block).await;
    let message = match result {
        Ok(_) => {
            EventMessage::success(EventType::UploadFileBlock, format!("{} upload success", file_block.id.unwrap()))
        }
        Err(e) => {
            error!("{} upload fail:{}", file_block.id.unwrap(), e.to_string());
            EventMessage::fail(EventType::UploadFileBlock, format!("{} upload fail:{}", file_block.id.unwrap(), e.to_string()))
        }
    };
    CONTEXT.event_message_manager.insert(&message).await?;
    Ok(())
}

/// 上传失败后的重试治理：指数退避，达到上限进入死信。
async fn mark_upload_failure(cloud_file_block: &mut CloudFileBlock) {
    let max_attempts = dotenvy::var("UPLOAD_MAX_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(UPLOAD_MAX_ATTEMPTS)
        .max(1);
    let next_attempt = cloud_file_block.retry_count + 1;
    cloud_file_block.retry_count = next_attempt;
    if next_attempt >= max_attempts {
        warn!(
            "block {} dead-lettered after {} attempts",
            cloud_file_block.id.unwrap(),
            next_attempt
        );
        cloud_file_block.status = FileStatus::Dead.into();
        cloud_file_block.next_retry_time = None;
    } else {
        cloud_file_block.status = FileStatus::UploadFail.into();
        let next = Local::now() + backoff_after(next_attempt);
        info!(
            "block {} retry {} after backoff until {}",
            cloud_file_block.id.unwrap(),
            next_attempt,
            next
        );
        cloud_file_block.next_retry_time = Some(next);
    }
}

async fn do_execute_one_block(cloud_file_block: &mut CloudFileBlock) -> ResponseResult<()> {
    let cloud_meta = CONTEXT
        .cloud_meta_manager
        .info(cloud_file_block.cloud_meta_id)
        .await?;
    let status: MetaStatus = cloud_meta.status.into();
    if status != MetaStatus::Enable {
        info!("storage status error {:?}", status);
        return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] status error {:?}", cloud_file_block.cloud_meta_id, status)));
    }
    let file_block_meta = CONTEXT
        .file_manager
        .file_block_meta_info_by_id(cloud_file_block.file_block_id)
        .await?;
    let mut file_block_meta = file_block_meta.unwrap();
    let cache_file = crate::fs::config::temp_path().await;

    let cache_file = format!("{cache_file}/{}", file_block_meta.file_part_id);
    info!("upload {}", cache_file);
    let result = File::open(cache_file.clone());
    if let Err(e) = result {
        let kink = e.kind();
        let info = match kink {
            ErrorKind::NotFound => {
                file_block_meta.status = FileStatus::FileNotExist.into();
                ErrorInfo::OTHER(2, format!("scan文件读取失败{}", e.kind()))
            }
            _ => {
                file_block_meta.status = FileStatus::FileReadError.into();
                ErrorInfo::OTHER(2, format!("scan文件读取失败{}:{}", e.kind(), e.to_string()))
            }
        };
        CONTEXT
            .file_manager
            .update_file_block_meta(file_block_meta)
            .await?;
        return Err(info);
    }
    let mut file = result?;
    let mut body: Vec<u8> = Vec::new();
    let result = file.read_to_end(&mut body);
    if let Err(e) = result {
        file_block_meta.status = FileStatus::FileReadError.into();
        let result = CONTEXT
            .file_manager
            .update_file_block_meta(file_block_meta)
            .await;
        return match result {
            Ok(_) => {
                Err(ErrorInfo::OTHER(2, format!("文件读取失败{}:{}", e.kind(), e.to_string())))
            }
            Err(e) => {
                Err(e)
            }
        };
    }

    let result = storage_facade()
        .upload_content(&file_block_meta, &mut cloud_file_block.clone(), &cloud_meta, &body)
        .await;
    let res = match result {
        Ok(cr) => {
            cloud_file_block.status = FileStatus::UploadSuccess.into();
            cloud_file_block.cloud_file_id = cr.cloud_file_id;
            cloud_file_block.cloud_file_hash = Some(file_block_meta.part_hash);
            cloud_file_block.retry_count = 0;
            cloud_file_block.next_retry_time = None;
            Ok(())
        }
        Err(e) => {
            mark_upload_failure(cloud_file_block).await;
            Err(ErrorInfo::OTHER(0, format!("上传文件{cache_file}:{}", e.to_string())))
        }
    };
    cloud_file_block.update_time = Local::now();
    let result = CONTEXT.cloud_file_block_manager.update_by_status(&cloud_file_block, FileStatus::Uploading).await;
    match result {
        Ok(count) => {
            match count {
                0 => {
                    if cloud_file_block.cloud_file_id.is_some() {
                        let temp = TempCloudFile::new(cloud_file_block.cloud_meta_id.clone(), cloud_file_block.cloud_file_id.clone().unwrap());
                        let result = CONTEXT.temp_cloud_file_manager.add(&temp).await;
                        match result {
                            Ok(_) => {}
                            Err(e) => {
                                error!("临时文件数据库更新失败:{}",e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Err(e) => {
            return Err(ErrorInfo::OTHER(0, format!("文件块{}更新文件块状态失败: {}", cloud_file_block.id.unwrap(), e.to_string())))
        }
    }
    res
}
