use log::error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio_cron_scheduler::{Job, JobScheduler};

use persistence::FileStatus;
use storage::STORAGE_FACADE;

use crate::task::clean_cloud_file::clean_cloud_file;
use crate::task::clean_recode::clean_recode;
use crate::task::mark_clean::mark_clean;
use crate::task::re_balance::re_balance;
use crate::task::refresh_token::refresh_token;
use crate::task::reset::reset;
use crate::task::scan::scan;

pub mod clean_cloud_file;
mod scan;
mod reset;
mod re_balance;
mod refresh_token;
mod mark_clean;
mod clean_recode;

pub async fn task(sched: &JobScheduler) {
    let locked = Job::new_async("* * * * * *", move |_uuid, _l| clean_cloud_file_task()).unwrap();
    sched.add(locked).await.unwrap();
    let semaphore = Arc::new(Semaphore::new(16));

    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| {
        scan_task(semaphore.clone())
    },
    ).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/10 * * * * *", |_uuid, _l| reset_task()).unwrap();
    sched.add(locked).await.unwrap();

    let locked = Job::new_async("0 0 * * * *", move |_uuid, _l| refresh_quota_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| re_balance_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0 * * * * *", move |_uuid, _l| refresh_token_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/5 * * * * *", move |_uuid, _l| mark_clean_task()).unwrap();
    sched.add(locked).await.unwrap();

    let locked = Job::new_async("*/5 * * * * *", move |_uuid, _l| clean_deleted_task()).unwrap();
    sched.add(locked).await.unwrap();
}

fn clean_cloud_file_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean_cloud_file().await;
        }
    })
}

fn mark_clean_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            error!("=========================mark_clean_task start==================================");
            mark_clean().await;
            error!("=========================mark_clean_task end==================================");
        }
    })
}

fn scan_task(semaphore: Arc<Semaphore>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = scan(semaphore.clone()).await;
            if let Err(e) = result {
                log::error!("re_balance error: {}", e);
            }
        }
    })
}

fn reset_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = reset(FileStatus::Uploading, FileStatus::Init, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
            let result = reset(FileStatus::UploadFail, FileStatus::Init, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
            let result = reset(FileStatus::Cleaning, FileStatus::WaitClean, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
            let result = reset(FileStatus::CleanFail, FileStatus::WaitClean, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
        }
    })
}

fn refresh_quota_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = STORAGE_FACADE.write().await.refresh_quota().await;
            if let Err(e) = result {
                log::error!("refresh_quota error: {}", e);
            }
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

fn refresh_token_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = refresh_token().await;
            if let Err(e) = result {
                log::error!("refresh_token error: {}", e);
            }
        }
    })
}
fn clean_deleted_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean_recode().await;
        }
    })
}
