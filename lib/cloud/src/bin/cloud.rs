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
const HOST_PORT: u32 = 11111;
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
    // let facade = StorageFacade::new();
    // let facade = Arc::new(Mutex::new(facade));
    let mut sched = JobScheduler::new().await.unwrap();
    task(&sched).await;
    sched.start().await.unwrap();
    // let facade = Arc::clone(&facade);
    let server = run_web(plugin_arc.clone()).await;
    let cache_file = String::from(DEFAULT_TEMP_PATH);

    let nfs = CloudNFS::new(&cache_file);
    info!("starting nfs server");
    let listener = NFSTcpListener::bind(&format!("0.0.0.0:{HOST_PORT}"), nfs)
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

// #[cfg(windows)]
// fn sigint_handler(server_handle: ServerHandle) {
//     let (tx, rx) = channel();
//     ctrlc::set_handler(move || {
//         tx.send(()).expect("failed to set Ctrl-C handler");
//         rt::System::new().block_on(server_handle.stop(true));
//         info!("stop")
//     }).expect("failed to set Ctrl-C handler");
//
//     info!("File system is mounted, press Ctrl-C to unmount.");
//     rx.recv().unwrap();
// }

// #[cfg(not(windows))]
// fn sigint_handler(server_handle: ServerHandle) {
// use log::error;
// let signals = signal_hook::iterator::Signals::new(&[
//     signal_hook::consts::SIGINT,
//     signal_hook::consts::SIGTERM,
// ]);
// if let Err(e) = signals {
//     error!("{}", e);
//     return;
// }
// let mut signals = signals.unwrap();
// for sig in signals.forever() {
//     info!("Received signal {:?}", sig);
//     rt::System::new().block_on(server_handle.stop(true));
//     info!("rt stopped");
//     break;
// }
// }
