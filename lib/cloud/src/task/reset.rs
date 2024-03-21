use std::time::Duration;

use log::info;
use rbatis::rbdc::datetime::DateTime;

use persistence::FileStatus;
use service::CONTEXT;

pub(crate) async fn reset(status: FileStatus, sub: u64) {
    info!("reset status:{:?} before {}s", status, sub);
    let mut update_time = DateTime::now();
    update_time =  update_time.sub(Duration::from_secs(sub));
    // info!("reset status:{} update_time:{}", status, update_time);
    let cloud_file_block = CONTEXT.cloud_file_block_manager.select_by_status(status.into(), update_time).await;
    // let cloud_file_block = CloudFileBlock::select_by_status(pool!(), status.into(), update_time).await.unwrap();
    info!("select from {:?} to reset,size:{}",  status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.status = FileStatus::Init.into();
        file_block.update_time = DateTime::now();
        CONTEXT.cloud_file_block_manager.update_by_status(&file_block,
                                                          file_block.id.unwrap(),
                                                          status.into()).await.unwrap().rows_affected;

    }
}
