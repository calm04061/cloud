use api::ResponseResult;
use chrono::{Local};
use log::{error, info};
use std::ops::Sub;
use std::time::Duration;

use persistence::FileStatus;
use service::CONTEXT;

pub(crate) async fn reset(origin_status: FileStatus, target_status: FileStatus, sub: u64) -> ResponseResult<()> {
    let update_time = Local::now();
    let update_time = update_time.sub(Duration::from_secs(sub));
    let cloud_file_block = CONTEXT.cloud_file_block_manager.select_by_status(origin_status, update_time).await?;
    info!("select from {:?} to reset,size:{}",  origin_status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.status = target_status.into();
        file_block.update_time = Local::now();
        let result = CONTEXT.cloud_file_block_manager.update_by_status(&file_block,
                                                                       origin_status).await;
        match result {
            Ok(_) => {}
            Err(e) => {
                error!("reset error:{}", e);
            }
        }
    }
    Ok(())
}
