use std::time::Duration;
use log::info;
use rbatis::rbdc::datetime::DateTime;
use persistence::{CloudFileBlock, FileStatus};

use crate::pool;

pub(crate) async fn reset(status: FileStatus, sub: u64) {
    info!("reset status:{:?} before {}s", status, sub);
    let mut update_time = DateTime::now();
    update_time =  update_time.sub(Duration::from_secs(sub));
    // info!("reset status:{} update_time:{}", status, update_time);
    let cloud_file_block = CloudFileBlock::select_by_status(pool!(), status.into(), update_time).await.unwrap();
    info!("select from {:?} to reset,size:{}",  status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.status = FileStatus::Init.into();
        file_block.update_time = DateTime::now();
        // info!("update {} status from {} to {}", file_block.id.unwrap(), status, 1);
        CloudFileBlock::update_by_status(
            pool!(),
            &file_block,
            file_block.id.unwrap(),
            status.into(),
        )
            .await
            .unwrap()
            .rows_affected;
    }
}
