use std::fs::File;
use std::io::{ErrorKind, Read};
use std::sync::Arc;

use log::{error, info};
use rbatis::rbdc::datetime::DateTime;
use tokio::sync::{Mutex, Semaphore};
use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::{CloudFileBlock, EventMessage, EventType, FileStatus, MetaStatus};
use service::{CONTEXT};
use service::database::meta::{CloudMetaManager, FileManager};

use crate::storage::storage_facade::StorageFacade;


pub(crate) async fn scan(semaphore: Arc<Semaphore>, facade: Arc<Mutex<StorageFacade>>) {
    info!("scan");
    let cloud_file_block = CONTEXT.cloud_file_block_manager.select_to_upload().await;
    // let cloud_file_block = CloudFileBlock::select_to_upload(pool!()).await.unwrap();
    info!("{} block to upload", cloud_file_block.len());
    for mut file_block in cloud_file_block {
        let origin_status = file_block.status;
        file_block.update_time = DateTime::now();
        file_block.status = FileStatus::Uploading.into();
        let count = CONTEXT.cloud_file_block_manager.update_by_status(&file_block, file_block.id.unwrap(), origin_status)
            .await.unwrap().rows_affected;
        if count == 0 {
            info!("block {} unable lock by db", file_block.id.unwrap());
            continue;
        }
        let semaphore = semaphore.clone();
        let arc = Arc::clone(&facade);

        tokio::spawn(async move {//创建任务
            let arc = Arc::clone(&arc);
            let _semaphore_permit = semaphore.acquire().await.unwrap(); // 通过信号量控制并发
            info!("upload block {} to cloud", file_block.id.unwrap());
            let result = do_execute_one_block(&mut file_block, arc).await;
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
        });
    }
}

async fn do_execute_one_block(file_block: &mut CloudFileBlock, facade: Arc<Mutex<StorageFacade>>) -> ResponseResult<()> {
    let cloud_meta = CONTEXT
        .cloud_meta_manager
        .info(file_block.cloud_meta_id)
        .await;
    if let None = cloud_meta {
        return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] not found", file_block.cloud_meta_id)));
    }
    let cloud_meta = cloud_meta.unwrap();
    let status: MetaStatus = cloud_meta.status.into();
    if status != MetaStatus::Enable {
        info!("storage status error {:?}", status);
        return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] status error {:?}", file_block.cloud_meta_id, status)));
    }
    let mut file_block_meta = CONTEXT
        .file_manager
        .file_block_meta_info_by_id(file_block.file_block_id)
        .await
        .unwrap();
    let cache_file = dotenvy::var("TEMP_PATH").unwrap_or(String::from("/var/lib/storage/temp"));

    let cache_file = format!("{cache_file}/{}",  file_block_meta.file_part_id);
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

    let mut cloud = facade.lock().await;
    let result = cloud
        .upload_content(&file_block_meta, &cloud_meta, &body)
        .await;
    return match result {
        Ok(cr) => {
            file_block.status = FileStatus::UploadSuccess.into();
            file_block.cloud_file_id = Some(cr.file_id);
            file_block.update_time = DateTime::now();
            file_block.cloud_file_hash = Some(file_block_meta.part_hash);
            CONTEXT.cloud_file_block_manager.update_by_status(
                &file_block,
                file_block.id.unwrap(),
                FileStatus::Uploading.into(),
            )
                .await?;
            Ok(())
        }
        Err(e) => {
            file_block.status = FileStatus::UploadFail.into();
            file_block.update_time = DateTime::now();
            CONTEXT.cloud_file_block_manager.update_by_status(
                &file_block,
                file_block.id.unwrap(),
                FileStatus::Uploading.into(),
            )
                .await?;
            Err(ErrorInfo::OTHER(0, format!("上传文件{cache_file}:{}", e.to_string())))
        }
    };
}
