use crate::database::meta::{FileStatus};
use crate::domain::table::tables::CloudFileBlock;
use crate::pool;
use quartz_sched::SchedulerHandle;
use rbatis::rbdc::datetime::DateTime;

pub(crate) struct Reset {
}

impl Reset {
    pub(crate) fn new() -> Self {
        Reset {
        }
    }
}

impl quartz_sched::Job for Box<Reset> {
    fn execute(&self, _engine: Option<SchedulerHandle>) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                reset(FileStatus::Uploading.into()).await;
                reset(FileStatus::UploadFail.into()).await;
            });
    }

    fn description(&self) -> String {
        String::from("Reset")
    }

    fn key(&self) -> i64 {
        4
    }
}

async fn reset(status: i8) {
    let update_time = DateTime::now();
    let timestamp = update_time.unix_timestamp();
    let update_time = DateTime::from_timestamp(timestamp - (30 * 10));
    let cloud_file_block = CloudFileBlock::select_by_status(pool!(), status, update_time).await.unwrap();
    // info!("select from {} to reset,size:{}",  status, cloud_file_block.len());
    for mut file_block in cloud_file_block {
        file_block.status = FileStatus::Init.into();
        file_block.update_time= DateTime::now();
        // info!("update {} status from {} to {}", file_block.id.unwrap(), status, 1);
        CloudFileBlock::update_by_status(
            pool!(),
            &file_block,
            file_block.id.unwrap(),
            status,
        )
            .await
            .unwrap()
            .rows_affected;
    }
}
