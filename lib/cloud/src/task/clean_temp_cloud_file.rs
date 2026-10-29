// 从clean.rs文件中提取的Rust代码，用于清理文件和云文件块

use api::error::ErrorInfo;
use api::ResponseResult;
use chrono::{Local};
use log::{debug, error};
use persistence::{FileStatus, TempCloudFile};
use service::CONTEXT;
use std::ops::Sub;
use std::sync::Arc;
use std::time::Duration;
use storage::STORAGE_FACADE;
use tokio::sync::Mutex;

pub async fn clean_temp_cloud_file(worker_id: Arc<Mutex<Option<i64>>>) {
    debug!("start clean");
    let result = query_file(worker_id).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return;
    }

    let result = result.unwrap();
    for mut temp_cloud_file in result {
        let _result = do_clean_file_block(&mut temp_cloud_file).await;

    }
    debug!("clean finish");
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

    let result = STORAGE_FACADE.delete_temp_file(&file_block_meta).await;
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


pub(crate) async fn reset_status(origin_status: FileStatus, target_status: FileStatus, sub: u64) -> ResponseResult<()> {
    // info!("reset status:{:?} before {}s", origin_status, sub);
    let update_time = Local::now();
    let update_time = update_time.sub(Duration::from_secs(sub));
    // info!("reset status:{:?} update_time:{}", origin_status, update_time);
    let cloud_file_block = CONTEXT.temp_cloud_file_manager.select_by_status(origin_status, update_time).await?;
    // let cloud_file_block = CloudFileBlock::select_by_status(pool!(), status.into(), update_time).await.unwrap();
    // info!("select from {:?} to reset,size:{}",  origin_status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.update_time = Local::now();
        let result = CONTEXT.temp_cloud_file_manager.update_status(&file_block,
                                                                       target_status).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                error!("reset error:{}", e);
            }
        }
    }
    Ok(())
}