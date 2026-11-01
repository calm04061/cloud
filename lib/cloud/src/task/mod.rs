use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::{Mutex, Semaphore};
use tokio_cron_scheduler::{Job, JobScheduler};

use persistence::FileStatus;
use storage::storage_facade;

use crate::task::clean_cloud_file::clean_cloud_file;
use crate::task::clean_temp_cloud_file::{clean_temp_cloud_file, reset_status};
use crate::task::re_balance::re_balance;
use crate::task::refresh_token::refresh_token;
use crate::task::reset::reset;
use crate::task::scan::scan;

pub(crate) mod clean_cloud_file;
mod scan;
mod reset;
mod re_balance;
mod refresh_token;
mod clean_temp_cloud_file;
mod evict_cache;

pub async fn task(sched: &JobScheduler) {
    let file_worker_id: Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    let temp_file_worker_id: Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));

    let locked = Job::new_async("5 * * * * *", move |_uuid, _l| clean_cloud_file_task(file_worker_id.clone())).unwrap();
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
    let locked = Job::new_async("0/10 * * * * *", move |_uuid, _l| refresh_token_task()).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/20 * * * * *", move |_uuid, _l| clean_temp_cloud_file_task(temp_file_worker_id.clone())).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0 0/5 * * * *", |_uuid, _l| evict_cache_task()).unwrap();
    sched.add(locked).await.unwrap();
}

fn clean_cloud_file_task(worker_id: Arc<Mutex<Option<i64>>>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean_cloud_file(worker_id.clone()).await;
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
            let result = reset_status(FileStatus::CleanFail, FileStatus::WaitClean, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
            let result = reset_status(FileStatus::Cleaning, FileStatus::WaitClean, 60 * 10).await;
            if let Err(e) = result {
                log::error!("reset error: {}", e);
            }
        }
    })
}

fn refresh_quota_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let result = storage_facade().refresh_quota().await;
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
fn clean_temp_cloud_file_task(worker_id: Arc<Mutex<Option<i64>>>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean_temp_cloud_file(worker_id).await;
        }
    })
}

fn evict_cache_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin(async {
        crate::task::evict_cache::evict_cache().await;
    })
}

