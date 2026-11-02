extern crate core;

use std::error::Error;
use std::sync::atomic::Ordering;
use std::time::Duration;

use cloud::fs::nfs::cloud_nfs::CloudNFS;
use cloud::fs::vfs::IN_FLIGHT_WRITES;
use cloud::fs::{log_effective_config, temp_path};
use dotenvy::dotenv;
use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use tokio_cron_scheduler::JobScheduler;

use cloud::task::task;
use cloud::web::run_web;
use plugin_manager::PLUGIN_MANAGER;
use service::CONTEXT;

const DEFAULT_NFS_BIND_ADDR: &str = "127.0.0.1:11111";

/// 工作线程数：`.env` 的 `TOKIO_WORKER_THREADS` > CPU 核心数。
/// 不再固定 50 线程，避免在小内存设备（如树莓派）上浪费资源。
fn build_runtime() -> tokio::runtime::Runtime {
    let workers = dotenvy::var("TOKIO_WORKER_THREADS")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4));
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env不存在");
    let runtime = build_runtime();
    runtime.block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let result = log4rs::init_file("log4rs.yaml", Default::default()); //.unwrap();
    match result {
        Ok(_) => {}
        Err(e) => {
            panic!("log4rs初始化失败:{}", e);
        }
    }
    info!("spawning thread for server");
    let plugin_arc = PLUGIN_MANAGER.get_plugins();
    cloud::repository::init_storage_repositories();
    CONTEXT.upgrade().await;
    log_effective_config().await;

    let mut sched = JobScheduler::new().await.unwrap();
    task(&sched).await;
    sched.start().await.unwrap();
    // let facade = Arc::clone(&facade);
    let server = run_web(plugin_arc.clone()).await;
    let cache_file = temp_path().await;

    let nfs_bind_addr = dotenvy::var("NFS_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_NFS_BIND_ADDR.to_string());
    let nfs = CloudNFS::new(&cache_file);
    info!("starting nfs server on {}", nfs_bind_addr);
    let listener = NFSTcpListener::bind(&nfs_bind_addr, nfs)
        .await
        .unwrap();
    let nfs_handle = tokio::spawn(async move {
        if let Err(e) = listener.handle_forever().await {
            log::error!("nfs server stopped with error: {}", e);
        }
    });

    server.await.expect("http server error");
    info!("server stopped");
    info!("stopping scheduler");
    sched.shutdown().await.unwrap();
    info!("scheduler stopped");
    info!("stopping nfs server");
    nfs_handle.abort();
    let _ = nfs_handle.await;
    info!("nfs server stopped");
    // 等待在途 VFS 写入排空，避免退出时把未上传完成的块静默丢弃
    drain_in_flight_writes(Duration::from_secs(30)).await;
    Ok(())
}

/// 等待所有在途 VFS 写入完成；超时则记录剩余数量后退出，
/// 未完成的块行停留在 Uploading/Init，下次启动由 reset/scan 任务接续。
async fn drain_in_flight_writes(timeout: Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    while IN_FLIGHT_WRITES.load(Ordering::SeqCst) > 0 {
        if tokio::time::Instant::now() >= deadline {
            let remaining = IN_FLIGHT_WRITES.load(Ordering::SeqCst);
            log::warn!(
                "shutdown timeout: {} in-flight writes not drained; \
                 their blocks stay queued and will be re-uploaded on next start",
                remaining
            );
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    info!("all in-flight writes drained");
}
