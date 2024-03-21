// 从clean.rs文件中提取的Rust代码，用于清理文件和云文件块

use log::{debug, error, warn};
use rbatis::rbdc::datetime::DateTime;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::{FileBlockMeta, FileMeta, FileStatus};
use service::CONTEXT;
use service::database::meta::FileManager;
use storage::STORAGE_FACADE;

pub async fn clean() {
    debug!("start clean");
    let cache_file = dotenvy::var("TEMP_PATH").unwrap_or_else(|_| String::from("/var/lib/storage/temp"));
    let ten = chrono::Local::now().timestamp_millis() - 10 * 1000;
    if let Ok(file_metas) = CONTEXT.file_manager.list_deleted_file(ten).await {
        debug!("clean file_meta:{}", file_metas.len());
        for mut file_meta in file_metas {
            if let Err(e) = clean_file_block(&mut file_meta, &cache_file).await {
                error!("Failed to clean file block: {}", e);
            }
        }
    } else {
        error!("Failed to retrieve deleted files");
    }
    debug!("clean finish");
}

async fn clean_file_block(file_meta: &mut FileMeta, cache_file: &str) -> ResponseResult<()>  {
    let file_block_metas = CONTEXT
        .file_manager
        .file_block_meta(file_meta.id.unwrap())
        .await;
    if file_block_metas.is_empty() {
        debug!("file_block_metas is empty");
        file_meta.status = FileStatus::WaitClean.into();
        if let Err(e) = CONTEXT.file_meta_manager.update_meta(&file_meta).await {
            error!("Failed to update file meta: {}", e);
        }
        return Ok(());
    }
    debug!("file_block_metas size:{}", file_block_metas.len());
    for mut file_block_meta in file_block_metas {
        let size = delete_cloud_file_block(file_meta, &file_block_meta).await?;
        if size == 0 {
            let local_cache_file = format!("{}/{}", cache_file, file_block_meta.file_part_id);
            if let Err(e) = std::fs::remove_file(&local_cache_file) {
                warn!("Failed to remove local cache file: {}", e);
            }
            file_block_meta.deleted = 1;
            if let Err(e) = CONTEXT.file_block_meta_manager.update_file_block_meta(&mut file_block_meta).await {
                error!("Failed to update file block meta: {}", e);
            }
            continue;
        }
    }
    Ok(())
}

async fn delete_cloud_file_block(file_meta: &mut FileMeta, file_block_meta: &FileBlockMeta) -> ResponseResult<usize> {
    let cloud_file_blocks = CONTEXT.cloud_file_block_manager.select_by_file_block_id(file_block_meta.id.unwrap()).await?;
    let size = cloud_file_blocks.len();

    for mut cloud_file_block in cloud_file_blocks {
        let id = cloud_file_block.id.unwrap();
        let db_status = cloud_file_block.status;
        let cleaning: i8 = FileStatus::Cleaning.into();
        if db_status == cleaning {
            let time = cloud_file_block.update_time.clone();
            let timestamp = time.unix_timestamp();
            let now = DateTime::now();
            let now = now.unix_timestamp();
            if now - timestamp > 1000 {
                cloud_file_block.status = FileStatus::WaitClean.into();
                if let Err(e) = CONTEXT.cloud_file_block_manager.update_by_status(&cloud_file_block, id, FileStatus::Cleaning.into()).await {
                    error!("Failed to update cloud file block status: {}", e);
                }
            }
            continue;
        }
        let result = CONTEXT.cloud_file_block_manager.update_by_status(&cloud_file_block, id, FileStatus::Cleaning.into())
            .await;
        if let Err(e) = result {
            error!("Failed to update cloud file block status: {}", e);
            continue;
        }
        let result = result.unwrap();
        if result.rows_affected == 0 {
            continue;
        }

        let result = STORAGE_FACADE.write().await.delete(&cloud_file_block).await;
        if let Ok(())= result{
            debug!("删除云文件成功");
            cloud_file_block.deleted = 1;
            CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await;
            continue;
        }
        let e = result.err().unwrap();
        match e {
            ErrorInfo::FileNotFound(_) => {
                cloud_file_block.deleted = 1;
                cloud_file_block.status = FileStatus::Cleaned.into();
                CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await;
            }
            ErrorInfo::NoneCloudFileId(_) => {
                cloud_file_block.deleted = 1;
                cloud_file_block.status = FileStatus::Cleaned.into();
                CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await;
            }
            _ => {
                cloud_file_block.status = FileStatus::CleanFail.into();
                error!("删除云文件失败:{},{}", file_meta.name, e);
            }
        }
    }
    Ok(size)
}

