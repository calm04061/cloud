use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use tokio::sync::{Mutex, RwLock};

type FileLockMap = RwLock<HashMap<i64, Arc<Mutex<()>>>>;

static FILE_LOCKS: OnceLock<FileLockMap> = OnceLock::new();

fn file_locks() -> &'static FileLockMap {
    FILE_LOCKS.get_or_init(Default::default)
}

pub(crate) async fn file_lock(file_meta_id: i64) -> Arc<Mutex<()>> {
    let lock = file_locks()
        .read()
        .await
        .get(&file_meta_id)
        .cloned();
    match lock {
        Some(lock) => lock,
        None => {
            let mut locks = file_locks().write().await;
            locks
                .entry(file_meta_id)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        }
    }
}
