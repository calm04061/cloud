mod common;

use persistence::meta::FileBlockMeta;
use service::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use service::meta::file::file_meta::SimpleFileMetaManager;
use service::meta::file::SimpleFileManager;
use service::meta::FileManager;

fn new_block_meta(file_meta_id: i64) -> FileBlockMeta {
    FileBlockMeta {
        id: None,
        block_index: 0,
        file_part_id: "part-0".to_string(),
        update_time: chrono::Local::now(),
        file_modify_time: 0,
        file_meta_id,
        deleted: 0,
        part_hash: "hash-1".to_string(),
        status: 0,
    }
}

/// 并发保存同一 (file_meta_id, block_index)：
/// 唯一索引 + on conflict 事务必须保证只存活一条块记录，
/// 且云端副本行（cloud_file_block）恰好创建一次。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_save_file_block_meta_dedupes_and_creates_replicas() {
    let (pool, path) = common::test_pool().await;
    common::apply_migrations(&pool).await;

    let cloud_meta_id = common::insert_enabled_cloud_meta(&pool, "cloud-1").await;
    let file_meta_id = common::insert_file_meta(&pool, "f.bin", 1, 0).await;

    let manager = std::sync::Arc::new(SimpleFileManager::new(
        pool.clone(),
        SimpleFileMetaManager::new(pool.clone()),
        SimpleFileBlockMetaManager::new(pool.clone()),
    ));
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let manager = manager.clone();
        tasks.push(tokio::spawn(async move {
            manager.save_file_block_meta(new_block_meta(file_meta_id)).await
        }));
    }
    for task in tasks {
        task.await
            .unwrap()
            .expect("concurrent save must succeed under on-conflict handling");
    }

    let (live_blocks,): (i64,) = sqlx::query_as(
        "select count(*) from file_block_meta where file_meta_id = ? and deleted = 0",
    )
    .bind(file_meta_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(live_blocks, 1, "exactly one live block meta must survive");

    let (replicas,): (i64,) = sqlx::query_as(
        "select count(*) from cloud_file_block where file_block_id = 1 and cloud_meta_id = ? and deleted = 0",
    )
    .bind(cloud_meta_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        replicas, 1,
        "replica row must be created exactly once per cloud account"
    );

    pool.close().await;
    common::cleanup(&path).await;
}

/// 删除目录必须原子地软删除全部后代：
/// 任一环节失败都不能留下半删除状态。
#[tokio::test]
async fn delete_file_meta_soft_deletes_whole_directory() {
    let (pool, path) = common::test_pool().await;
    common::apply_migrations(&pool).await;

    let dir_id = common::insert_file_meta(&pool, "parent-dir", 1, 2).await;
    let child_1 = common::insert_file_meta(&pool, "child-1", dir_id, 2).await;
    let child_2 = common::insert_file_meta(&pool, "child-2", dir_id, 0).await;

    let manager = SimpleFileManager::new(
        pool.clone(),
        SimpleFileMetaManager::new(pool.clone()),
        SimpleFileBlockMetaManager::new(pool.clone()),
    );
    manager.delete_file_meta(dir_id).await.unwrap();

    for id in [dir_id, child_1, child_2] {
        let (deleted,): (i8,) = sqlx::query_as("select deleted from file_meta where id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(deleted, 1, "file_meta {id} must be soft-deleted");
    }

    let live = common::count(&pool, "select count(*) from file_meta where deleted = 0 and id > 1").await;
    assert_eq!(live, 0, "no live descendants may remain");

    pool.close().await;
    common::cleanup(&path).await;
}
