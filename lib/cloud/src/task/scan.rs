use std::fs::File;
use std::io::{ErrorKind, Read};
use std::sync::Arc;
use chrono::Utc;
use log::{error, info};
use tokio::sync::Semaphore;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::{CloudFileBlock, EventMessage, EventType, FileStatus, MetaStatus};
use service::meta::{CloudMetaManager, FileManager};
use service::CONTEXT;
use storage::STORAGE_FACADE;

pub(crate) async fn scan(semaphore: Arc<Semaphore>) -> ResponseResult<()> {
    info!("scan");
    let cloud_file_block_result = CONTEXT.cloud_file_block_manager.select_to_upload().await;
    if let Err(e) = cloud_file_block_result {
        error!("select_to_upload error {:?}", e);
        return Ok(());
    }
    let cloud_file_blocks = cloud_file_block_result.unwrap();
    info!("{} block to upload", cloud_file_blocks.len());
    for file_block in cloud_file_blocks {
        let semaphore = semaphore.clone();
        tokio::spawn(async move {
            process_block(file_block, semaphore.clone()).await.unwrap();
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
    file_block.update_time = Utc::now();
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
    CONTEXT.event_message_manager.insert(&message).await.unwrap();
    return Ok(());
}

async fn do_execute_one_block(file_block: &mut CloudFileBlock) -> ResponseResult<()> {
    let cloud_meta = CONTEXT
        .cloud_meta_manager
        .info(file_block.cloud_meta_id)
        .await?;
    let status: MetaStatus = cloud_meta.status.into();
    if status != MetaStatus::Enable {
        info!("storage status error {:?}", status);
        return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] status error {:?}", file_block.cloud_meta_id, status)));
    }
    let file_block_meta = CONTEXT
        .file_manager
        .file_block_meta_info_by_id(file_block.file_block_id)
        .await
        .unwrap();
    let mut file_block_meta = file_block_meta.unwrap();
    let cache_file = dotenvy::var("TEMP_PATH").unwrap_or(String::from("/var/lib/storage/temp"));

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
    let mut file = result.unwrap();
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

    let result = STORAGE_FACADE.write().await
        .upload_content(&file_block_meta, &cloud_meta, &body)
        .await;
    let res = match result {
        Ok(cr) => {
            file_block.status = FileStatus::UploadSuccess.into();
            file_block.cloud_file_id = Some(cr.file_id);
            file_block.cloud_file_hash = Some(file_block_meta.part_hash);
            Ok(())
        }
        Err(e) => {
            file_block.status = FileStatus::UploadFail.into();
            Err(ErrorInfo::OTHER(0, format!("上传文件{cache_file}:{}", e.to_string())))
        }
    };
    file_block.update_time = Utc::now();
    CONTEXT.cloud_file_block_manager.update_by_status(&file_block, FileStatus::Uploading).await?;
    res
}
