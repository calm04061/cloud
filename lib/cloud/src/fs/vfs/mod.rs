use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

mod blocks;
mod cache;
mod lock;
mod namespace;
#[cfg(test)]
mod tests;

pub(crate) use lock::file_lock;

pub const DEFAULT_TEMP_PATH: &str = "/var/lib/storage/temp";

pub const CLOUD_FILE_BLOCK_SIZE: usize = 4194304 * 4;

pub(crate) static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
pub(crate) static CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
/// 正在进行的 VFS 写入数（含未上传完成的块）
pub static IN_FLIGHT_WRITES: AtomicU64 = AtomicU64::new(0);

/// 本地缓存命中率统计：(命中, 未命中)
pub fn cache_stats() -> (u64, u64) {
    (
        CACHE_HITS.load(Ordering::SeqCst),
        CACHE_MISSES.load(Ordering::SeqCst),
    )
}

// 16384 * 256; //1024k
#[derive(Clone, Debug)]
pub(crate) struct VirtualFileSystem {
    pub(crate) inner: Inner,
}

#[derive(Clone)]
pub(crate) struct Inner {
    pub(crate) cache_file: String,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.cache_file)
            .finish()
    }
}

impl VirtualFileSystem {
    pub(crate) fn new(cache_file: &str) -> VirtualFileSystem {
        VirtualFileSystem {
            inner: Inner {
                cache_file: String::from(cache_file),
            },
        }
    }
}
