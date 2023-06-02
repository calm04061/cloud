mod clean;
mod refresh_quota;
mod scan;
mod reset;

use quartz_sched::Scheduler;
use std::time::Duration;

use crate::task::clean::Clean;
use crate::task::refresh_quota::RefreshQuota;
use crate::task::reset::Reset;
use crate::task::scan::Scan;

pub fn task_init(sched: &Scheduler<8>) {
    let cache_file = dotenv::var("TEMP_PATH").unwrap_or(String::from("/var/lib/storage/temp"));
    let cache_file = cache_file.as_str();
    sched.schedule_task(quartz_sched::schedule_task_every(
        Duration::from_secs(1),
        Box::new(Clean::new(cache_file)),
    ));
    sched.schedule_task(quartz_sched::schedule_task_every(
        Duration::from_secs(5),
        Box::new(Scan::new(cache_file)),
    ));
    sched.schedule_task(quartz_sched::schedule_task_every(
        Duration::from_secs(5),
        Box::new(Reset::new()),
    ));
    sched.schedule_task(quartz_sched::schedule_task_every(
        Duration::from_secs(60 * 60 * 24),
        Box::new(RefreshQuota::new()),
    ));
}
