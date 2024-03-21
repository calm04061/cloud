use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio_cron_scheduler::{Job, JobScheduler};

use persistence::FileStatus;
use storage::STORAGE_FACADE;

use crate::task::clean::clean;
use crate::task::re_balance::re_balance;
use crate::task::reset::reset;
use crate::task::scan::scan;

pub mod clean;
mod scan;
mod reset;
mod re_balance;


pub async fn task(sched: &JobScheduler) {
    let locked = Job::new_async("* * * * * *", move |_uuid, _l| clean_task()).unwrap();
    sched.add(locked).await.unwrap();
    let semaphore = Arc::new(Semaphore::new(10));

    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| {
        scan_task(semaphore.clone())
    },
    ).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/5 * * * * *", |_uuid, _l| reset_task()).unwrap();
    sched.add(locked).await.unwrap();

    let locked = Job::new_async("0 0 * * * *", move |_uuid, _l| refresh_quota_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| re_balance_task()).unwrap();
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
            reset(FileStatus::Uploading.into(), 60 * 5).await;
            reset(FileStatus::UploadFail.into(), 60).await;
        }
    })
}

fn refresh_quota_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            STORAGE_FACADE.write().await.refresh_quota().await;
        }
    })
}

fn re_balance_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = re_balance().await;
            if let Err(e) = result {
                log::error!("re_balance error: {}", e);
            }
        }
    })
}
