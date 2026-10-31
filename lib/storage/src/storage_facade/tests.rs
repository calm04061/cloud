use super::*;
use crate::model::Quota;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// 记录同时处于临界区的请求数与历史最大并发
struct ConcurrencyTracker {
    active: AtomicUsize,
    max: AtomicUsize,
}

impl ConcurrencyTracker {
    fn new() -> Self {
        ConcurrencyTracker {
            active: AtomicUsize::new(0),
            max: AtomicUsize::new(0),
        }
    }
    fn enter(&self) {
        let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max.fetch_max(current, Ordering::SeqCst);
    }
    fn exit(&self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
    fn max(&self) -> usize {
        self.max.load(Ordering::SeqCst)
    }
}

/// 模拟慢速网络 IO 的存储适配器
struct MockStorage {
    delay: Duration,
    global_tracker: Arc<ConcurrencyTracker>,
    account_tracker: Arc<ConcurrencyTracker>,
}

#[async_trait::async_trait]
impl Storage for MockStorage {
    async fn upload_content(
        &mut self,
        _file_block: &FileBlockMeta,
        cloud_file_block: &mut CloudFileBlock,
        _content: &Vec<u8>,
        _cloud_meta: &CloudMeta,
    ) -> ResponseResult<CloudFileBlock> {
        self.global_tracker.enter();
        self.account_tracker.enter();
        tokio::time::sleep(self.delay).await;
        self.account_tracker.exit();
        self.global_tracker.exit();
        Ok(cloud_file_block.clone())
    }

    async fn delete(&mut self, _cloud_file_id: &str, _cloud_meta: &CloudMeta) -> ResponseResult<()> {
        Ok(())
    }

    async fn content(&mut self, _cloud_file_id: &str, _cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        Ok(Bytes::new())
    }

    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        Ok(Quota {
            total: 0,
            used: 0,
            remaining: 0,
        })
    }

    async fn refresh_token(&mut self, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        Ok(String::new())
    }

    fn authorize(&self, _server: &str, _id: i32) -> ResponseResult<String> {
        Ok(String::new())
    }

    async fn callback(&self, _server: String, _code: String, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        Ok(String::new())
    }
}

fn mock_cloud_meta(id: i32) -> CloudMeta {
    CloudMeta {
        id: Some(id),
        name: format!("account-{}", id),
        auth: None,
        last_work_time: None,
        data_root: None,
        status: 0,
        deleted: 0,
        cloud_type: 0,
        total_quota: None,
        used_quota: None,
        remaining_quota: None,
        extra: None,
        expires_in: None,
    }
}

fn mock_file_block(file_meta_id: i64, block_index: i64) -> FileBlockMeta {
    FileBlockMeta {
        id: None,
        block_index,
        file_part_id: format!("part-{}-{}", file_meta_id, block_index),
        update_time: chrono::Local::now(),
        file_modify_time: 0,
        file_meta_id,
        deleted: 0,
        part_hash: "hash".to_string(),
        status: 0,
    }
}

fn mock_cloud_file_block(cloud_meta_id: i32) -> CloudFileBlock {
    CloudFileBlock {
        id: None,
        file_block_id: 1,
        cloud_meta_id,
        cloud_file_id: Some("cloud-file".to_string()),
        cloud_file_hash: Some("hash".to_string()),
        status: 0,
        deleted: 0,
        create_time: chrono::Local::now(),
        update_time: chrono::Local::now(),
    }
}

fn facade_with_accounts(
    accounts: &[(i32, Arc<ConcurrencyTracker>)],
    global: Arc<ConcurrencyTracker>,
) -> StorageFacade {
    let mut account_holder = HashMap::new();
    for (id, tracker) in accounts {
        let storage: CloudStorage = Arc::new(Mutex::new(MockStorage {
            delay: Duration::from_millis(100),
            global_tracker: global.clone(),
            account_tracker: tracker.clone(),
        }));
        account_holder.insert(*id, storage);
    }
    StorageFacade {
        root: OnceCell::new(),
        plugin_holder: HashMap::new(),
        account_holder: RwLock::new(account_holder),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn uploads_to_different_accounts_run_in_parallel() {
    let tracker_a = Arc::new(ConcurrencyTracker::new());
    let tracker_b = Arc::new(ConcurrencyTracker::new());
    let global = Arc::new(ConcurrencyTracker::new());
    let facade = Arc::new(facade_with_accounts(
        &[(1, tracker_a.clone()), (2, tracker_b.clone())],
        global.clone(),
    ));

    let mut tasks = Vec::new();
    for account_id in [1, 1, 2, 2] {
        let facade = facade.clone();
        let block = mock_file_block(100, 0);
        let mut cloud_file_block = mock_cloud_file_block(account_id);
        let meta = mock_cloud_meta(account_id);
        tasks.push(tokio::spawn(async move {
            facade
                .upload_content(&block, &mut cloud_file_block, &meta, &vec![0u8; 8])
                .await
                .expect("upload should succeed");
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }

    // 同一账号共享一把锁，最多 1 个请求在途（同账号串行）
    assert_eq!(tracker_a.max(), 1, "same account uploads must serialize");
    assert_eq!(tracker_b.max(), 1, "same account uploads must serialize");
    // 两个账号同时各有 1 个请求在途，证明跨账号并行、慢请求互不阻塞
    assert_eq!(
        global.max(),
        2,
        "uploads to different accounts must run in parallel"
    );
}
