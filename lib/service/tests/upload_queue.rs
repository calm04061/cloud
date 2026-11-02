use chrono::{Duration as ChronoDuration, Local};
mod common;
use common::*;
use service::cloud_file_block_manager::CloudFileBlockManager;
use service::meta::file::file_block_meta::SimpleFileBlockMetaManager;

/// #15：上传队列的选取语义——退避窗口、死信隔离、活跃租约不被重复处理。
#[tokio::test]
async fn select_to_upload_honors_backoff_dead_letter_and_claim() {
    let (pool, path) = test_pool().await;
    apply_migrations(&pool).await;

    let block_manager = CloudFileBlockManager::new(pool.clone());
    let fbm_manager = SimpleFileBlockMetaManager::new(pool.clone());

    // 造 file_block_meta，使 join 能匹配（part_hash 与副本行一致）
    let fbm_id = insert_file_block_meta(&pool, "part-a", 0, "hash-a").await;

    // (file_block_id, cloud_meta_id) 唯一：每行用不同 cloud_meta_id 区分场景
    let insert_block = |cloud_meta_id: i32, status: i8, retry: i32, next_retry: Option<chrono::DateTime<Local>>| {
        let pool = pool.clone();
        async move {
            sqlx::query("insert into cloud_file_block (file_block_id, cloud_meta_id, cloud_file_hash, cloud_file_id, status, deleted, create_time, update_time, retry_count, next_retry_time) values (?, ?, 'hash-a', null, ?, 0, datetime(), datetime(), ?, ?)")
                .bind(fbm_id)
                .bind(cloud_meta_id)
                .bind(status)
                .bind(retry)
                .bind(next_retry)
                .execute(&pool)
                .await
                .unwrap();
        }
    };


    // Init 新入队：可见
    insert_block(1, 1, 0, None).await;
    // UploadFail 退避未到期：不可见
    insert_block(2, 4, 1, Some(Local::now() + ChronoDuration::minutes(30))).await;
    // UploadFail 退避到期：可见
    insert_block(3, 4, 2, Some(Local::now() - ChronoDuration::seconds(5))).await;
    // 死信：永不可见
    insert_block(4, 10, 10, None).await;
    // 活跃租约（Uploading，1 分钟前）：不可见，避免多实例重复处理
    insert_block(5, 2, 0, None).await;
    // 已删除：不可见
    insert_block(6, 1, 0, None).await;
    sqlx::query("update cloud_file_block set deleted = 1 where cloud_meta_id = 6")
        .execute(&pool)
        .await
        .unwrap();

    let selected = block_manager
        .select_to_upload(Local::now())
        .await
        .unwrap();
    let statuses: Vec<i8> = selected.iter().map(|b| b.status).collect();
    assert!(statuses.contains(&1), "Init 行必须可见");
    assert!(statuses.contains(&4), "退避到期的 UploadFail 行必须可见");
    assert_eq!(
        statuses.iter().filter(|s| **s == 4).count(),
        1,
        "退避未到期的 UploadFail 行不可见"
    );
    assert!(!statuses.contains(&10), "死信行必须被隔离");
    assert!(!statuses.contains(&2), "活跃租约（Uploading）不可被重复处理");

    // 语义检查：manager.update_by_status 的 CAS 抢占
    // （Init→Uploading 成功一次后，第二次从 Init 抢占必然 count=0）
    let mut claim = selected.first().unwrap().clone();
    claim.status = 2;
    let first = block_manager.update_by_status(&claim, persistence::FileStatus::Init).await.unwrap();
    let second = block_manager.update_by_status(&claim, persistence::FileStatus::Init).await.unwrap();
    assert_eq!(first, 1);
    assert_eq!(second, 0, "同一行不允许被两次 CAS 抢占");

    drop(fbm_manager);
    pool.close().await;
    cleanup(&path).await;
}

/// 与 file_block_meta 表结构对齐的最小插入助手（hash 指定，便于 join 匹配）。
async fn insert_file_block_meta(pool: &service::DbPool, part_id: &str, index: i64, hash: &str) -> i32 {
    sqlx::query("insert into file_block_meta (file_part_id, block_index, update_time, file_modify_time, file_meta_id, deleted, part_hash, status) values (?, ?, datetime(), 0, 1, 0, ?, 3)")
        .bind(part_id)
        .bind(index)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
    let (id,): (i32,) = sqlx::query_as("select last_insert_rowid()")
        .fetch_one(pool)
        .await
        .unwrap();
    id
}
