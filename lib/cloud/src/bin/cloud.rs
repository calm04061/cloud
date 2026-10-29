extern crate core;

use std::error::Error;

use cloud::fs::nfs::cloud_nfs::CloudNFS;
use cloud::fs::vfs::DEFAULT_TEMP_PATH;
use dotenvy::dotenv;
use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use tokio::main;
use tokio_cron_scheduler::JobScheduler;

use cloud::task::task;
use cloud::web::run_web;
use plugin_manager::PLUGIN_MANAGER;
use service::CONTEXT;

const DEFAULT_NFS_BIND_ADDR: &str = "127.0.0.1:11111";
#[main(flavor = "multi_thread", worker_threads = 50)]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env不存在");
    let result = log4rs::init_file("log4rs.yaml", Default::default()); //.unwrap();
    match result {
        Ok(_) => {}
        Err(e) => {
            panic!("log4rs初始化失败:{}", e);
        }
    }
    info!("spawning thread for server");
    let plugin_arc = PLUGIN_MANAGER.get_plugins();
    CONTEXT.upgrade().await;

    let mut sched = JobScheduler::new().await.unwrap();
    task(&sched).await;
    sched.start().await.unwrap();
    // let facade = Arc::clone(&facade);
    let server = run_web(plugin_arc.clone()).await;
    let cache_file = String::from(DEFAULT_TEMP_PATH);

    let nfs_bind_addr = dotenvy::var("NFS_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_NFS_BIND_ADDR.to_string());
    let nfs = CloudNFS::new(&cache_file);
    info!("starting nfs server on {}", nfs_bind_addr);
    let listener = NFSTcpListener::bind(&nfs_bind_addr, nfs)
        .await
        .unwrap();
    tokio::spawn(async move {
        listener.handle_forever().await.unwrap();
    });

    server.await.expect("TODO: panic message");
    info!("server stopped");
    info!("stopping scheduler");
    sched.shutdown().await.unwrap();
    info!("scheduler stopped");
    Ok(())
}
