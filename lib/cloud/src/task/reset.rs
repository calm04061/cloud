use std::ops::Sub;
use std::time::Duration;
use chrono::{Local, Utc};
use api::ResponseResult;
use log::{error, info};

use persistence::FileStatus;
use service::CONTEXT;

pub(crate) async fn reset(origin_status: FileStatus, target_status: FileStatus, sub: u64) -> ResponseResult<()> {
    // info!("reset status:{:?} before {}s", origin_status, sub);
    let update_time = Local::now();
    let time = update_time.to_utc();
    let update_time = time.sub(Duration::from_secs(sub));
    // info!("reset status:{} update_time:{}", status, update_time);
    let cloud_file_block = CONTEXT.cloud_file_block_manager.select_by_status(origin_status, update_time).await?;
    // let cloud_file_block = CloudFileBlock::select_by_status(pool!(), status.into(), update_time).await.unwrap();
    info!("select from {:?} to reset,size:{}",  origin_status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.status = target_status.into();
        file_block.update_time = Utc::now();
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
