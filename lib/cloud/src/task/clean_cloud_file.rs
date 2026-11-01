use api::error::ErrorInfo;
use api::ResponseResult;
use chrono::{TimeDelta, Utc};
use log::{debug, error};
use persistence::meta::{FileBlockMeta, FileMeta};
use persistence::{CloudFileBlock, FileMetaType, FileStatus};
use service::meta::FileManager;
use service::CONTEXT;
use std::io::ErrorKind;
use std::ops::Sub;
use std::sync::Arc;
use storage::storage_facade;
use tokio::sync::Mutex;

pub async fn clean_cloud_file(worker_id: Arc<Mutex<Option<i64>>>) {
    debug!("start clean");
    let result = query_file(worker_id.clone()).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return;
    }
    let cache_file = crate::fs::config::temp_path().await;

    let result = result.unwrap();
    let mut failed = false;
    for file_meta in result {
        let result = do_clean_file_meta(&file_meta, &cache_file).await;
        if let Err(e) = result {
            error!("Failed to clean file meta: {}", e);
            // 释放抢占锁并回置 CleanFail，保证下一轮（游标归零后）可以重试，
            // 否则失败的文件永远停在 Cleaning，云端副本会永久残留
            let _ = CONTEXT
                .file_meta_manager
                .update_status_by_id(file_meta.id.unwrap(), FileStatus::CleanFail)
                .await;
            failed = true;
        }
    }
    // 本轮存在失败文件时，把游标归零，避免失败行落在游标之前被跳过
    if failed {
        let mut guard = worker_id.lock().await;
        *guard = Some(0);
    }
    debug!("clean finish");
}
async fn query_file(worker_id: Arc<Mutex<Option<i64>>>) -> ResponseResult<Vec<FileMeta>> {
    let delta = TimeDelta::seconds(60);
    let ten = Utc::now().sub(delta);
    // 抢占中的文件超过 10 分钟视为上一轮崩溃遗留，允许重新抢占
    let reclaim = Utc::now().sub(TimeDelta::minutes(10));
    let mut guard = worker_id.lock().await;
    let pre_id;
    if guard.is_none() {
        pre_id = 0;
    } else {
        pre_id = guard.unwrap();
    }

    let result = CONTEXT.file_manager.list_deleted_file(ten, pre_id, reclaim).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return Err(e);
    }
    let mut list = result?;
    if pre_id > 0 && list.is_empty() {
        let result = CONTEXT.file_manager.list_deleted_file(ten, 0, reclaim).await;
        if let Err(e) = result {
            error!("Failed to retrieve deleted files: {}", e);
            return Err(e);
        }
        list = result?;
    }
    if list.is_empty() {
        return Ok(vec![]);
    }
    let last = list.get(list.len() - 1).unwrap();
    let last = last.id.unwrap();
    *guard = Some(last);
    Ok(list)
}
///
/// 清理具体的一个文件
/// 1. 删除文件块
/// 2. 第一步全部成功,删除数据库记录
///
async fn do_clean_file_meta(file_meta: &FileMeta, cache_file: &String) -> ResponseResult<u64> {
    let count = CONTEXT.file_meta_manager.update_status(&file_meta, FileStatus::Cleaning).await?;
    if count == 0 {
        return Err(ErrorInfo::OTHER(1003, format!("文件{}抢占失败", file_meta.id.unwrap())));
    }
    //  抢锁
    let file_block_metas = CONTEXT.file_block_meta_manager.file_block_meta(file_meta.id.unwrap()).await?;
    let mut result = vec![];
    for file_block in file_block_metas {
        result.push(do_clean_file_block(&file_block, cache_file).await);
    }
    for row in result {
        if let Err(e) = row {
            return Err(e);
        }
    }
    if file_meta.file_type == FileMetaType::DIR.get_code() {
        let vec = CONTEXT.file_manager.list_all_by_parent(file_meta.id.unwrap()).await?;
        if vec.len() > 0 {
            return Ok(0);
        }
    }
    CONTEXT.file_meta_manager.delete_meta(file_meta).await
}
///
/// 删除文件块
/// 1. 删除云端文件块
/// 2. 删除本地文件
/// 3. 如果1,2都操作成功，删除数据库记录
///
async fn do_clean_file_block(file_block_meta: &FileBlockMeta, cache_file: &String) -> ResponseResult<u64> {
    let count = CONTEXT.file_block_meta_manager.update_status(&file_block_meta, FileStatus::Cleaning).await?;
    if count == 0 {
        return Err(ErrorInfo::OTHER(1003, format!("文件块{}抢占失败", file_block_meta.id.unwrap())));
    }
    let mut result = vec![];
    let cloud_file_blocks = CONTEXT.cloud_file_block_manager.select_by_file_block_id(file_block_meta.id.unwrap()).await?;
    for mut cloud_file_block in cloud_file_blocks {
        result.push(do_clean_cloud_file(&mut cloud_file_block).await);
    }
    let local_cache_file = format!("{}/{}", cache_file, file_block_meta.file_part_id);
    if let Err(e) = std::fs::remove_file(&local_cache_file) {
        match e.kind() {
            ErrorKind::NotFound => {
                result.push(Ok(1));
            }
            _ => {
                result.push(Err(ErrorInfo::new_string(1003, e.to_string())));
            }
        }
    }
    for row in result {
        if let Err(e) = row {
            return Err(e);
        }
    }
    CONTEXT.file_block_meta_manager.delete_by_id(file_block_meta.id.unwrap()).await
}

async fn do_clean_cloud_file(cloud_file_block: &mut CloudFileBlock) -> ResponseResult<u64> {
    let count = CONTEXT.cloud_file_block_manager.update_status(&cloud_file_block, FileStatus::Cleaning).await?;
    if count == 0 {
        return Err(ErrorInfo::OTHER(1003, format!("云文件块{}抢占失败", cloud_file_block.id.unwrap())));
    }
    let result = storage_facade().delete(&cloud_file_block).await;
    if let Ok(()) = result {
        debug!("删除云文件成功");
        return CONTEXT.cloud_file_block_manager.delete_by_id(cloud_file_block.id.unwrap()).await;
    }
    let e = result.err().unwrap();
    match e {
        ErrorInfo::FileNotFound(_) => {
            CONTEXT.cloud_file_block_manager.delete_by_id(cloud_file_block.id.unwrap()).await
        }
        ErrorInfo::NoneCloudFileId(_) => {
            CONTEXT.cloud_file_block_manager.delete_by_id(cloud_file_block.id.unwrap()).await
        }
        _ => {
            cloud_file_block.status = FileStatus::CleanFail.into();
            error!("删除云文件失败:{}", e);
            CONTEXT.cloud_file_block_manager.update(&cloud_file_block).await?;
            Err(ErrorInfo::OTHER(1003, format!("云文件块{}删除失败:{}", cloud_file_block.id.unwrap(), e)))
        }
    }
}
