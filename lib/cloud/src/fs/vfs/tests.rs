use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// 同一 inode 的并发写必须被 per-inode 锁串行化（ARCH-004）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn file_lock_serializes_same_inode() {
    let active = Arc::new(AtomicUsize::new(0));
    let max = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();
    for _ in 0..4 {
        let active = active.clone();
        let max = max.clone();
        tasks.push(tokio::spawn(async move {
            let lock = file_lock(101).await;
            let _guard = lock.lock().await;
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            max.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(30)).await;
            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }

    assert_eq!(
        max.load(Ordering::SeqCst),
        1,
        "critical sections on the same inode must never overlap"
    );
}

/// 不同 inode 之间不共享锁，可以并行。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn file_lock_allows_different_inodes() {
    let active = Arc::new(AtomicUsize::new(0));
    let max = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();
    for ino in [201u64, 202u64] {
        let active = active.clone();
        let max = max.clone();
        tasks.push(tokio::spawn(async move {
            let lock = file_lock(ino as i64).await;
            let _guard = lock.lock().await;
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            max.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(80)).await;
            active.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }

    assert!(
        max.load(Ordering::SeqCst) >= 2,
        "different inodes must be able to enter critical sections concurrently"
    );
}
