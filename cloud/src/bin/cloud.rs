extern crate core;

use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;

use actix_web::dev::ServerHandle;
use actix_web::rt;
use log::{info};

use cloud::task::task_init;
use cloud::web::run_web;

fn main() -> Result<(), Box<dyn Error>> {
    let result = log4rs::init_file("log4rs.yaml", Default::default()); //.unwrap();
    match result {
        Ok(_) => {}
        Err(e) => {
            panic!("log4rs初始化失败:{}", e);
        }
    }

    let (tx, rx) = channel();
    info!("spawning thread for server");
    thread::spawn(move || {
        let server_future = run_web(tx);
        rt::System::new().block_on(server_future)
    });

    let server_handle = rx.recv().unwrap();
    let mut sched: quartz_sched::Scheduler = quartz_sched::Scheduler::new();
    info!("task_start");
    sched.start();
    info!("task_init");
    task_init(&sched);
    info!("task_init end");
    sigint_handler(server_handle);
    info!("stop sched");
    sched.stop();
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
    use log::{error};
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
