// 从clean.rs文件中提取的Rust代码，用于清理文件和云文件块

use std::ops::{Sub};
use chrono::{TimeDelta, Utc};
use log::{debug, error};

use service::meta::FileManager;
use service::CONTEXT;

pub async fn mark_clean() {
    debug!("start clean");
    let delta = TimeDelta::seconds(10);
    let ten = Utc::now().sub(delta);
    let result = CONTEXT.file_manager.list_deleted_file(ten).await;
    if let Err(e) = result {
        error!("Failed to retrieve deleted files: {}", e);
        return;
    }
    let file_metas = result.unwrap();
    debug!("clean file_meta:{}", file_metas.len());
    for mut file_meta in file_metas {
        if let Err(e) = CONTEXT.file_manager.mark_clean_file_block(&mut file_meta).await {
            error!("Failed to clean file block: {}", e);
        }
    }
    debug!("clean finish");
}


