mod common;

/// 验证 0.0.3 迁移创建的 partial unique index：
/// 并发/重复创建同名文件必须在数据库层被拒绝。
#[tokio::test]
async fn unique_index_rejects_duplicate_live_file_name() {
    let (pool, path) = common::test_pool().await;
    common::apply_migrations(&pool).await;

    let insert = |name: &str| {
        sqlx::query("insert into file_meta (name, parent_id, file_type, mode, gid, uid, file_length, status, deleted, create_time, update_time) values (?, 1, 0, 644, 1000, 1000, 0, 0, 0, datetime(), datetime())")
            .bind(name)
    };
    insert("same.txt").execute(&pool).await.unwrap();

    let duplicate = insert("same.txt").execute(&pool).await;
    assert!(
        duplicate.is_err(),
        "second live file with same (parent_id, name) must violate unique index"
    );

    // 软删除后允许重新创建同名文件
    sqlx::query("update file_meta set deleted = 1 where name = 'same.txt'")
        .execute(&pool)
        .await
        .unwrap();
    insert("same.txt").execute(&pool).await.unwrap();

    pool.close().await;
    common::cleanup(&path).await;
}

/// 同一文件的同一 block_index 只允许一条存活记录。
#[tokio::test]
async fn unique_index_rejects_duplicate_block_meta() {
    let (pool, path) = common::test_pool().await;
    common::apply_migrations(&pool).await;

    let insert = |index: i64| {
        sqlx::query("insert into file_block_meta (block_index, file_part_id, update_time, file_modify_time, deleted, file_meta_id, part_hash, status) values (?, 'part', datetime(), 0, 0, 1, 'hash', 0)")
            .bind(index)
    };
    insert(0).execute(&pool).await.unwrap();

    let duplicate = insert(0).execute(&pool).await;
    assert!(
        duplicate.is_err(),
        "second live block with same (file_meta_id, block_index) must violate unique index"
    );

    pool.close().await;
    common::cleanup(&path).await;
}

/// 同一文件块的同一云账号只允许一条副本记录。
#[tokio::test]
async fn unique_index_rejects_duplicate_cloud_file_block() {
    let (pool, path) = common::test_pool().await;
    common::apply_migrations(&pool).await;

    let insert = |cloud_meta_id: i32| {
        sqlx::query("insert into cloud_file_block (file_block_id, cloud_meta_id, cloud_file_id, cloud_file_hash, status, deleted, create_time, update_time) values (1, ?, 'cf-1', 'hash', 0, 0, datetime(), datetime())")
            .bind(cloud_meta_id)
    };
    insert(1).execute(&pool).await.unwrap();

    let duplicate = insert(1).execute(&pool).await;
    assert!(
        duplicate.is_err(),
        "second replica with same (file_block_id, cloud_meta_id) must violate unique index"
    );

    pool.close().await;
    common::cleanup(&path).await;
}
