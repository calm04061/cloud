use super::*;
use std::sync::{Arc, Once};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use service::meta::FileManager;

/// 集成测试共享一个进程级临时 SQLite：首个用到的测试进程先设置 DATABASE_URL，
/// 再初始化 CONTEXT（connect_lazy_with，首个查询前必须完成设置）。
static DB_INIT: Once = Once::new();
static DB_READY: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

async fn ctx() -> &'static service::ServiceContext {
    DB_INIT.call_once(|| {
        let dir = std::env::temp_dir().join(format!("cloud-vfs-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        std::env::set_var(
            "DATABASE_URL",
            format!("sqlite://{}?mode=rwc", dir.join("file.db").display()),
        );
        std::env::set_var(
            "TEMP_PATH",
            dir.join("temp").to_string_lossy().to_string(),
        );
    });
    let ctx = &*service::CONTEXT;
    DB_READY
        .get_or_init(|| async {
            ctx.upgrade().await;
        })
        .await;
    ctx
}

fn temp_cache_dir(tag: &str) -> String {
    let dir = std::env::temp_dir()
        .join(format!("cloud-vfs-cache-{}", std::process::id()))
        .join(tag);
    std::fs::create_dir_all(&dir).expect("create cache dir");
    dir.to_string_lossy().to_string()
}

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

/// create_dir 后 path_meta 能解析回同一目录，类型为 DIR。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_dir_and_path_meta_roundtrip() {
    let _ = ctx().await;
    let vfs = VirtualFileSystem::new(&temp_cache_dir("roundtrip"));

    let created = vfs.create_dir("/vfs-roundtrip").await.unwrap();
    assert_eq!(
        created.file_type,
        persistence::FileMetaType::DIR.get_code()
    );

    let back = vfs.path_meta("/vfs-roundtrip").await.unwrap();
    assert_eq!(created.id, back.id);
    assert_eq!(
        back.file_type,
        persistence::FileMetaType::DIR.get_code()
    );
}

/// rename_path 移动文件到新父目录，源路径消失；重名目标报 FileAlreadyExist。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rename_path_moves_file() {
    let _ = ctx().await;
    let vfs = VirtualFileSystem::new(&temp_cache_dir("rename"));

    let dir_a = vfs.create_dir("/vfs-rename-a").await.unwrap();
    let dir_b = vfs.create_dir("/vfs-rename-b").await.unwrap();
    let created = vfs
        .create_file(dir_a.id.unwrap() as u64, "doc.txt", persistence::FileMetaType::FILE)
        .await
        .unwrap();
    let a_path = format!("/vfs-rename-a/doc.txt");

    vfs.rename_path(&a_path, "/vfs-rename-b/doc.txt").await.unwrap();

    let moved = vfs.path_meta("/vfs-rename-b/doc.txt").await.unwrap();
    assert_eq!(moved.id, created.id);
    assert!(vfs.path_meta(&a_path).await.is_err());

    // 目标已存在时必须报冲突
    let conflict = vfs
        .create_file(dir_a.id.unwrap() as u64, "clash", persistence::FileMetaType::FILE)
        .await
        .unwrap();
    vfs.rename_path(&format!("/vfs-rename-a/clash"), "/vfs-rename-b/doc.txt")
        .await
        .expect_err("renaming onto an existing name must fail");
    assert!(conflict.id.is_some());
}

/// setattr 透传 mode/uid/gid/mtime 并持久化。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn setattr_persists_attributes() {
    let _ = ctx().await;
    let vfs = VirtualFileSystem::new(&temp_cache_dir("setattr"));

    let dir = vfs.create_dir("/vfs-setattr").await.unwrap();
    let created = vfs
        .create_file(dir.id.unwrap() as u64, "attr.bin", persistence::FileMetaType::FILE)
        .await
        .unwrap();
    let mtime = 1_700_000_000_000_i64;

    let updated = vfs
        .setattr(created.id.unwrap() as u64, None, Some(0o644), Some(1000), Some(1000), Some(mtime))
        .await
        .unwrap();

    assert_eq!(updated.mode, 0o644);
    assert_eq!(updated.uid, 1000);
    assert_eq!(updated.gid, 1000);
    let back = vfs.file_info(created.id.unwrap() as u64).await.unwrap();
    assert_eq!(back.mode, 0o644);
    assert_eq!(back.update_time.timestamp_millis(), mtime);
}

/// 小文件写读往返：写入走本地缓存块，读回同样数据。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_read_roundtrip_single_block() {
    let _ = ctx().await;
    let vfs = VirtualFileSystem::new(&temp_cache_dir("io"));

    let dir = vfs.create_dir("/vfs-io").await.unwrap();
    let created = vfs
        .create_file(dir.id.unwrap() as u64, "small.bin", persistence::FileMetaType::FILE)
        .await
        .unwrap();
    let ino = created.id.unwrap() as u64;
    let data: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();

    let written = vfs.write(ino, 0, &data).await.unwrap();
    assert_eq!(written as usize, data.len());

    let read_back = vfs.read(ino, 0, data.len() as u32).await.unwrap();
    assert_eq!(read_back, data);

    // file_length 保存的是最后一次写路径覆盖的长度
    let meta = vfs.file_info(ino).await.unwrap();
    assert_eq!(meta.file_length, data.len() as i64);
}

/// 跨块写入：超过单个块大小（16MiB）的数据拆两个块，读取时聚合。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn write_read_roundtrip_cross_block() {
    let _ = ctx().await;
    let vfs = VirtualFileSystem::new(&temp_cache_dir("cross"));

    let dir = vfs.create_dir("/vfs-cross").await.unwrap();
    let created = vfs
        .create_file(dir.id.unwrap() as u64, "big.bin", persistence::FileMetaType::FILE)
        .await
        .unwrap();
    let ino = created.id.unwrap() as u64;

    let block = CLOUD_FILE_BLOCK_SIZE;
    let tail = block + 1024;
    let data: Vec<u8> = (0..tail).map(|i| ((i / 7) % 251) as u8).collect();

    let written = vfs.write(ino, 0, &data).await.unwrap();
    assert_eq!(super::IN_FLIGHT_WRITES.load(Ordering::SeqCst), 0, "写入结束后必须归零");
    assert_eq!(written as usize, data.len());

    let meta = vfs.file_info(ino).await.unwrap();
    assert_eq!(meta.file_length, tail as i64);

    let blocks = service::CONTEXT
        .file_manager
        .file_block_meta(created.id.unwrap())
        .await
        .unwrap();
    assert_eq!(blocks.len(), 2, "跨块写入应产生两个块");

    let read_back = vfs.read(ino, 0, data.len() as u32).await.unwrap();
    assert_eq!(read_back.len(), data.len());
    assert_eq!(read_back, data);
}
