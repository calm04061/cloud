use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio_cron_scheduler::{Job, JobScheduler};
use persistence::FileStatus;
use crate::storage::storage_facade::StorageFacade;

use crate::task::clean::{clean};
use crate::task::re_balance::re_balance;
use crate::task::reset::{reset};
use crate::task::scan::scan;

pub mod clean;
mod scan;
mod reset;
mod re_balance;


pub async fn task(sched: &JobScheduler, facade: Arc<Mutex<StorageFacade>>) {
    let clean_facade = Arc::clone(&facade);
    let locked = Job::new_async("* * * * * *", move |_uuid, _l| clean_task(Arc::clone(&clean_facade))).unwrap();
    sched.add(locked).await.unwrap();
    let semaphore = Arc::new(Semaphore::new(10));
    let scan_facade = Arc::clone(&facade);

    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| {
        scan_task(semaphore.clone(), Arc::clone(&scan_facade))
    },
    ).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("*/5 * * * * *", |_uuid, _l| reset_task()).unwrap();
    sched.add(locked).await.unwrap();
    let refresh_quota_facade = Arc::clone(&facade);

    let locked = Job::new_async("0 0 * * * *", move |_uuid, _l| refresh_quota_task(Arc::clone(&refresh_quota_facade))).unwrap();
    sched.add(locked).await.unwrap();
    let locked = Job::new_async("0/5 * * * * *", move |_uuid, _l| re_balance_task()).unwrap();
    sched.add(locked).await.unwrap();
}

fn clean_task(facade: Arc<Mutex<StorageFacade>>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            clean(Arc::clone(&facade)).await;
        }
    })
}

fn scan_task(semaphore: Arc<Semaphore>, facade: Arc<Mutex<StorageFacade>>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            scan(semaphore.clone(), Arc::clone(&facade)).await;
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

fn refresh_quota_task(facade: Arc<Mutex<StorageFacade>>) -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            let mut guard = facade.lock().await;
            guard.refresh_quota().await;
        }
    })
}

fn re_balance_task() -> Pin<Box<impl Future<Output=()> + Sized>> {
    Box::pin({
        async move {
            re_balance().await;
        }
    })
}
