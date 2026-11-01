use persistence::MetaStatus;
use service::DbPool;
use sqlx_sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static DB_SEQ: AtomicU32 = AtomicU32::new(0);

pub fn unique_db_path() -> std::path::PathBuf {
    let seq = DB_SEQ.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cloud-service-test-{}-{}-{}.db",
        std::process::id(),
        nanos,
        seq
    ))
}

pub async fn test_pool() -> (DbPool, std::path::PathBuf) {
    let path = unique_db_path();
    let options = SqliteConnectOptions::from_str(path.to_str().unwrap())
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(10));
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_lazy_with(options);
    (pool, path)
}

/// 按 0.0.1 -> 0.0.3 的顺序在全新数据库上执行全部迁移，
/// 与 ServiceContext::upgrade 的启动行为保持一致。
pub async fn apply_migrations(pool: &DbPool) {
    sqlx::raw_sql(include_str!("../../src/migrate/version/0.0.1.sql"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../src/migrate/version/0.0.2.sql"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!("../../src/migrate/version/0.0.3.sql"))
        .execute(pool)
        .await
        .unwrap();
}

pub async fn cleanup(path: &std::path::PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
}

pub async fn count(pool: &DbPool, sql: &'static str) -> i64 {
    let (value,): (i64,) = sqlx::query_as(sql).fetch_one(pool).await.unwrap();
    value
}

pub async fn insert_file_meta(pool: &DbPool, name: &str, parent_id: i64, file_type: i8) -> i64 {
    // 必须在同一连接上取 last_insert_rowid：连接池会复用任意连接，
    // 跨连接读取会拿到其他语句的插入 id
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("insert into file_meta (name, parent_id, file_type, mode, gid, uid, file_length, status, deleted, create_time, update_time) values (?, ?, ?, 644, 1000, 1000, 0, 0, 0, datetime(), datetime())")
        .bind(name)
        .bind(parent_id)
        .bind(file_type)
        .execute(&mut *conn)
        .await
        .unwrap();
    let (id,): (i64,) = sqlx::query_as("select last_insert_rowid()")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    id
}

pub async fn insert_enabled_cloud_meta(pool: &DbPool, name: &str) -> i32 {
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("insert into cloud_meta (name, auth, status, deleted, cloud_type) values (?, null, ?, 0, 0)")
        .bind(name)
        .bind::<i8>(MetaStatus::Enable.into())
        .execute(&mut *conn)
        .await
        .unwrap();
    let (id,): (i32,) = sqlx::query_as("select last_insert_rowid()")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    id
}
