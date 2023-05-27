use crate::storage::storage_facade::StorageFacade;
use quartz_sched::SchedulerHandle;

pub(crate) struct RefreshQuota {}

impl RefreshQuota {
    pub(crate) fn new() -> Self {
        RefreshQuota {}
    }
}

impl quartz_sched::Job for Box<RefreshQuota> {
    fn execute(&self, _engine: Option<SchedulerHandle>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut facade_cloud = StorageFacade::new();
                facade_cloud.refresh_quota().await;
            });
    }

    fn description(&self) -> String {
        String::from("刷新网盘容量")
    }

    fn key(&self) -> i64 {
        3
    }
}
