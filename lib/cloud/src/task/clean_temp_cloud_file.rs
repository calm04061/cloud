// 从clean.rs文件中提取的Rust代码，用于清理文件和云文件块

use api::error::ErrorInfo;
use api::ResponseResult;
use log::{debug, error, info};
use persistence::{FileStatus, TempCloudFile};
use service::CONTEXT;
use std::sync::Arc;
use storage::STORAGE_FACADE;
use tokio::sync::Mutex;

pub async fn clean_temp_cloud_file(worker_id: Arc<Mutex<Option<i64>>>) {
    info!("start clean");
    let result = query_file(worker_id).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return;
    }

    let result = result.unwrap();
    for mut temp_cloud_file in result {
        let _result = do_clean_file_block(&mut temp_cloud_file).await;

    }
    info!("clean finish");
}
async fn query_file(worker_id: Arc<Mutex<Option<i64>>>) -> ResponseResult<Vec<TempCloudFile>> {
    let mut guard = worker_id.lock().await;
    let pre_id;
    if guard.is_none() {
        pre_id = 0;
    } else {
        pre_id = guard.unwrap();
    }

    let result = CONTEXT.temp_cloud_file_manager.find_by_status(pre_id, FileStatus::WaitClean).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return Err(e);
    }
    let mut list = result?;
    if pre_id > 0 && list.is_empty() {
        let result = CONTEXT.temp_cloud_file_manager.find_by_status(0, FileStatus::WaitClean).await;
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
/// 删除文件块
/// 1. 删除云端文件块
/// 2. 删除本地文件
/// 3. 如果1,2都操作成功，删除数据库记录
///
async fn do_clean_file_block(file_block_meta: &mut TempCloudFile) -> ResponseResult<u64> {
    let count = CONTEXT.temp_cloud_file_manager.update_status(file_block_meta, FileStatus::Cleaning).await?;
    if count == 0 {
        return Err(ErrorInfo::OTHER(1003, format!("文件块{}抢占失败", file_block_meta.id.unwrap())));
    }

    let result = STORAGE_FACADE.write().await.delete_temp_file(&file_block_meta).await;
    if let Ok(()) = result {
        debug!("删除云文件成功");
        return CONTEXT.temp_cloud_file_manager.delete_by_id(file_block_meta.id.unwrap()).await;
    }
    let e = result.err().unwrap();
    match e {
        ErrorInfo::FileNotFound(_) => {
            CONTEXT.temp_cloud_file_manager.delete_by_id(file_block_meta.id.unwrap()).await
        }
        ErrorInfo::NoneCloudFileId(_) => {
            CONTEXT.temp_cloud_file_manager.delete_by_id(file_block_meta.id.unwrap()).await
        }
        _ => {
            // file_block_meta.status = FileStatus::CleanFail.into();
            error!("删除云文件失败:{}", e);
            CONTEXT.temp_cloud_file_manager.update_status(&file_block_meta, FileStatus::CleanFail).await?;
            Err(ErrorInfo::OTHER(1003, format!("临时文件块{}删除失败:{}", file_block_meta.id.unwrap(), e)))
        }
    }
}
