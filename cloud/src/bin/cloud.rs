extern crate core;

use std::error::Error;
use std::sync::mpsc::channel;

use actix_web::dev::ServerHandle;
use actix_web::rt;
use dotenv::dotenv;
use log::info;
use tokio::runtime::Builder;
use tokio_cron_scheduler::JobScheduler;
use cloud::task::task;

use cloud::web::run_web;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env不存在");
    let result = log4rs::init_file("log4rs.yaml", Default::default()); //.unwrap();
    match result {
        Ok(_) => {}
        Err(e) => {
            panic!("log4rs初始化失败:{}", e);
        }
    }

    let (tx, rx) = channel();
    info!("spawning thread for server");
    let runtime = Builder::new_multi_thread().
        enable_all().
        build().
        expect("create tokio runtime failed");
    runtime.spawn(async {
        run_web(tx).await.unwrap();
    });
    runtime.spawn(async {
        let sched = JobScheduler::new().await.unwrap();
        task(&sched).await;
        sched.start().await.unwrap();
    });

    let server_handle = rx.recv().unwrap();
    sigint_handler(server_handle);
    Ok(())
}

#[cfg(windows)]
fn sigint_handler(server_handle: ServerHandle) {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("failed to set Ctrl-C handler");
        rt::System::new().block_on(server_handle.stop(true));
        info!("stop")
    }).expect("failed to set Ctrl-C handler");

    info!("File system is mounted, press Ctrl-C to unmount.");
    rx.recv().unwrap();
}

#[cfg(not(windows))]
fn sigint_handler(server_handle: ServerHandle) {
    use log::error;
    let signals = signal_hook::iterator::Signals::new(&[
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ]);
    if let Err(e) = signals {
        error!("{}", e);
        return;
    }
    let mut signals = signals.unwrap();
    for sig in signals.forever() {
        info!("Received signal {:?}", sig);
        rt::System::new().block_on(server_handle.stop(true));
        info!("rt stopped");
        break;
    }
}
