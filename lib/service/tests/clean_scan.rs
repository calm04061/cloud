use chrono::{Duration as ChronoDuration, Utc};
mod common;
use common::*;
use service::meta::FileManager;
use service::meta::file::SimpleFileManager;
use service::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use service::meta::file::file_meta::SimpleFileMetaManager;

/// #3 / #15：清理扫描不得把失败的链路永久卡死。
///
/// - 抢占中的 Cleaning 行在宽限期内不可见（软锁）
/// - 超过 10 分钟的 Cleaning 行视为崩溃遗留，可重新抢占（归还不死锁）
/// - CleanFail 行立即可见，下一轮重试
/// - 宽限期（60 秒）内的软删除不可见，避免 VFS 删除落库前误清理
#[tokio::test]
async fn deleted_files_scan_releases_stuck_locks() {
    let (pool, path) = test_pool().await;
    apply_migrations(&pool).await;

    // 宽限期外的正常软删除：可见
    let (grace_ok,) = mark_deleted_with(&pool, "ok.txt", -300, 0).await;
    // 宽限期内：不可见
    let (grace_pending,) = mark_deleted_with(&pool, "late.txt", -30, 0).await;
    // 抢占中的 Cleaning，但仍在 10 分钟内：软锁生效，不可见
    let (locked_fresh,) = mark_deleted_with(&pool, "locked.txt", -120, 7).await;
    // 崩溃遗留的 Cleaning，超过 10 分钟：可重新抢占
    let (locked_stale,) = mark_deleted_with(&pool, "stale.txt", -3600, 7).await;
    // 上轮失败回置的 CleanFail：立即可见重试
    let (retryable,) = mark_deleted_with(&pool, "retry.txt", -120, 8).await;

    let manager = SimpleFileManager::new(pool.clone(), SimpleFileMetaManager::new(pool.clone()), SimpleFileBlockMetaManager::new(pool.clone()));
    let now = Utc::now();
    let mut ids = manager
        .list_deleted_file(now - ChronoDuration::seconds(60), 0, now - ChronoDuration::minutes(10))
        .await
        .unwrap()
        .into_iter()
        .map(|m| m.id.unwrap())
        .collect::<Vec<_>>();
    ids.sort();

    assert!(ids.contains(&grace_ok), "超过宽限期的软删除必须可见");
    assert!(ids.contains(&locked_stale), "阻塞超过 10 分钟的 Cleaning 必须可重新抢占（否则云端永久残留）");
    assert!(ids.contains(&retryable), "CleanFail 行必须可见以便下轮重试");
    assert!(!ids.contains(&grace_pending), "宽限期内的软删除不可见");
    assert!(!ids.contains(&locked_fresh), "宽限期内抢占中的 Cleaning 软锁必须生效");

    // 游标跳过的失败行也不会因 pre_id 永久丢失：pre_id 之后的行照常分页返回
    let first = ids.first().copied().unwrap();
    let next = manager
        .list_deleted_file(now - ChronoDuration::seconds(60), first, now - ChronoDuration::minutes(10))
        .await
        .unwrap();
    assert!(
        next.iter().all(|m| m.id.unwrap() > first),
        "分页必须严格按 id 递增"
    );

    pool.close().await;
    cleanup(&path).await;
}

/// 软删除某文件并把 update_time 回拨 seconds 秒、status 置为指定状态。
/// 时间用 chrono 绑定写入，与 list_deleted_file 的 chrono 绑定比较口径一致。
/// 返回 (文件 id,)。
async fn mark_deleted_with(pool: &service::DbPool, name: &str, seconds_ago: i64, status: i8) -> (i64,) {
    use chrono::Duration as ChronoDuration;
    let id = insert_file_meta(pool, name, 1, 0).await;
    sqlx::query("update file_meta set deleted = 1, status = ?, update_time = ? where id = ?")
        .bind(status)
        .bind(Utc::now() + ChronoDuration::seconds(seconds_ago))
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    (id,)
}
