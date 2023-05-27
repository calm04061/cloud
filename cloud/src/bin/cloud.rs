extern crate core;

use std::error::Error;
use std::sync::mpsc;
use std::thread;

use actix_web::dev::ServerHandle;
use actix_web::rt;
use log::{error, info};

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

    let (tx, rx) = mpsc::channel();
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term)).unwrap();
    while !term.load(Ordering::Relaxed) {}
    info!("Received signal ");
    rt::System::new().block_on(server_handle.stop(true));
}

#[cfg(not(windows))]
fn sigint_handler(server_handle: ServerHandle) {
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
