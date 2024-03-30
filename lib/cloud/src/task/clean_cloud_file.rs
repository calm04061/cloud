// 从clean.rs文件中提取的Rust代码，用于清理文件和云文件块

use log::{debug, error, warn};

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::meta::FileBlockMeta;
use persistence::{CloudFileBlock, FileStatus};
use service::CONTEXT;
use storage::STORAGE_FACADE;

pub async fn clean_cloud_file() {
    debug!("start clean");

    let cloud_file_blocks = CONTEXT.cloud_file_block_manager.select_by_status_limit(FileStatus::WaitClean, 10).await;
    if let Err(e) = cloud_file_blocks {
        error!("Failed to select file block meta by status limit: {}", e);
        return;
    }
    let cloud_file_blocks = cloud_file_blocks.unwrap();
    for mut cloud_file_block in cloud_file_blocks {
        let result = do_clean_cloud_file(&mut cloud_file_block).await;
        if let Err(e) = result {
            error!("Failed to clean cloud file: {}", e);
        }
    }
    let cache_file = dotenvy::var("TEMP_PATH").unwrap_or_else(|_| String::from("/var/lib/storage/temp"));

    let file_block_metas = CONTEXT.file_block_meta_manager.select_by_status_limit(FileStatus::WaitClean, 10).await;
    if let Err(e) = file_block_metas {
        error!("Failed to select file block meta: {}", e);
        return;
    }
    let file_block_metas = file_block_metas.unwrap();
    for mut file_block_meta in file_block_metas {
        let result = do_clean_file_block_file(&mut file_block_meta, &cache_file).await;
        if let Err(e) = result {
            error!("Failed to clean cloud file: {}", e);
        }
    }
}

async fn do_clean_file_block_file(file_block_meta: &mut FileBlockMeta, cache_file: &str) -> ResponseResult<()>{
    file_block_meta.status = FileStatus::Cleaning.into();

    let count = CONTEXT.file_block_meta_manager.update_by_status(&file_block_meta, FileStatus::WaitClean).await?;
    if count == 0 {
        return Ok(());
    }
    let local_cache_file = format!("{}/{}", cache_file, file_block_meta.file_part_id);
    if let Err(e) = std::fs::remove_file(&local_cache_file) {
        warn!("Failed to remove local cache file: {}", e);
    }
    file_block_meta.deleted = 1;
    file_block_meta.status = FileStatus::Cleaned.into();
    CONTEXT.file_block_meta_manager.update_file_block_meta(file_block_meta).await?;
    Ok(())
}

async fn do_clean_cloud_file(cloud_file_block: &mut CloudFileBlock) -> ResponseResult<()> {
    cloud_file_block.status = FileStatus::Cleaning.into();

    let count = CONTEXT.cloud_file_block_manager.update_by_status(&cloud_file_block, FileStatus::WaitClean).await?;
    if count == 0 {
        return Ok(());
    }
    let result = STORAGE_FACADE.write().await.delete(&cloud_file_block).await;
    if let Ok(()) = result {
        debug!("删除云文件成功");
        cloud_file_block.deleted = 1;
        cloud_file_block.status = FileStatus::Cleaned.into();
        CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await?;
        return Ok(());
    }
    let e = result.err().unwrap();
    match e {
        ErrorInfo::FileNotFound(_) => {
            cloud_file_block.deleted = 1;
            cloud_file_block.status = FileStatus::Cleaned.into();
        }
        ErrorInfo::NoneCloudFileId(_) => {
            cloud_file_block.deleted = 1;
            cloud_file_block.status = FileStatus::Cleaned.into();
        }
        _ => {
            cloud_file_block.status = FileStatus::CleanFail.into();
            error!("删除云文件失败:{}", e);
        }
    }
    CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await?;
    Ok(())
}
