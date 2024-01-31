use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::database::meta::FileStatus;
use crate::storage::storage_facade::StorageFacade;

use crate::task::clean::{clean};
use crate::task::reset::{reset};
use crate::task::scan::scan;

pub mod clean;
mod scan;
mod reset;


pub async fn task(sched: &JobScheduler) {
    let locked = Job::new_async("* * * * * *", |_uuid, _l| clean_task()).unwrap();
    sched.add(locked).await.unwrap();
    let semaphore = Arc::new(Semaphore::new(10));

    let locked = Job::new_async("0/5 * * * * *", move|_uuid, _l| {
        scan_task(semaphore.clone())
    }
    ).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/5 * * * * *", |_uuid, _l| reset_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0 0 * * * *", |_uuid, _l| refresh_quota_task()).unwrap();
    sched.add(locked).await.unwrap();
}

fn clean_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean().await;
        }
    })
}

fn scan_task(semaphore: Arc<Semaphore>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            scan(semaphore.clone()).await;
        }
    })
}

fn reset_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            reset(FileStatus::Uploading.into(), 30 * 10).await;
            reset(FileStatus::UploadFail.into(),10).await;
        }
    })
}

fn refresh_quota_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let mut facade_cloud = StorageFacade::new();
            facade_cloud.refresh_quota().await;
        }
    })
}
