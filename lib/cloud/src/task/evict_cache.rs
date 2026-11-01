use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime};

use api::ResponseResult;
use log::{error, info, warn};
use service::CONTEXT;

use crate::fs::vfs;

const DEFAULT_CACHE_MAX_GB: f64 = 20.0;
const DEFAULT_WATERMARK_PERCENT: u64 = 80;
/// 最近 N 分钟内修改过的缓存文件视为活跃上传，不参与淘汰
const ACTIVE_MINUTES: u64 = 10;

pub(crate) struct CacheEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

/// 缓存配额（字节）：`.env` 的 `CACHE_MAX_GB`，默认 20GB。
pub fn cache_max_bytes() -> u64 {
    dotenvy::var("CACHE_MAX_GB")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| *v > 0.0)
        .map(|gb| (gb * 1024.0 * 1024.0 * 1024.0) as u64)
        .unwrap_or((DEFAULT_CACHE_MAX_GB * 1024.0 * 1024.0 * 1024.0) as u64)
}

/// 水位告警阈值（占配额百分比）：`.env` 的 `CACHE_WATERMARK_PERCENT`，默认 80。
pub fn cache_watermark_percent() -> u64 {
    dotenvy::var("CACHE_WATERMARK_PERCENT")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|&v| (1..=100).contains(&v))
        .unwrap_or(DEFAULT_WATERMARK_PERCENT)
}

/// 尚未上传成功的块的本地缓存文件是唯一副本，不可淘汰。
async fn protected_part_ids() -> ResponseResult<HashSet<String>> {
    let rows = CONTEXT.file_block_meta_manager.protected_part_ids().await?;
    Ok(rows.into_iter().collect())
}

fn scan_cache_dir(dir: &Path) -> std::io::Result<Vec<CacheEntry>> {
    let mut entries = Vec::new();
    for item in std::fs::read_dir(dir)? {
        let item = item?;
        let metadata = item.metadata()?;
        if metadata.is_dir() {
            continue;
        }
        entries.push(CacheEntry {
            path: item.path(),
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }
    Ok(entries)
}

/// 选出需要淘汰的缓存条目：
/// 1. 排除受保护块（未上传成功的唯一副本）
/// 2. 排除活跃文件（mtime 晚于 active_before）
/// 3. 按 mtime 从旧到新淘汰，直到总量降到 target_bytes 以下
pub(crate) fn select_evictions(
    mut entries: Vec<CacheEntry>,
    current_total: u64,
    target_bytes: u64,
    protected: &HashSet<String>,
    active_before: SystemTime,
) -> Vec<CacheEntry> {
    if current_total <= target_bytes {
        return Vec::new();
    }
    entries.retain(|entry| {
        if entry.modified > active_before {
            return false;
        }
        match entry.path.file_name().and_then(|n| n.to_str()) {
            Some(name) => !protected.contains(name),
            None => false,
        }
    });
    entries.sort_by_key(|entry| entry.modified);

    let mut total = current_total;
    let mut evictions = Vec::new();
    for entry in entries {
        if total <= target_bytes {
            break;
        }
        total = total.saturating_sub(entry.size);
        evictions.push(entry);
    }
    evictions
}

fn evict_files(evictions: &[CacheEntry]) -> (u64, usize) {
    let mut freed = 0u64;
    let mut count = 0usize;
    for entry in evictions {
        match std::fs::remove_file(&entry.path) {
            Ok(()) => {
                freed += entry.size;
                count += 1;
            }
            Err(e) => {
                error!("淘汰缓存文件 {} 失败: {}", entry.path.display(), e);
            }
        }
    }
    (freed, count)
}

pub async fn evict_cache() {
    let temp_path = crate::fs::config::temp_path().await;
    let max_bytes = cache_max_bytes();
    let target_bytes = max_bytes / 2;

    // 保护名单查询失败时放弃本轮淘汰，宁可超配额也不误删唯一副本
    let protected = match protected_part_ids().await {
        Ok(protected) => protected,
        Err(e) => {
            error!("查询受保护块失败，跳过本轮缓存淘汰: {}", e);
            return;
        }
    };

    let temp_dir = temp_path.clone();
    let scan = tokio::task::spawn_blocking(move || scan_cache_dir(Path::new(&temp_dir))).await;
    let entries = match scan {
        Ok(Ok(entries)) => entries,
        Ok(Err(e)) => {
            error!("扫描缓存目录 {} 失败: {}", temp_path, e);
            return;
        }
        Err(e) => {
            error!("扫描缓存目录任务失败: {}", e);
            return;
        }
    };

    let total_before: u64 = entries.iter().map(|e| e.size).sum();
    let (hits, misses) = vfs::cache_stats();
    let total_requests = hits + misses;
    let hit_rate = if total_requests == 0 {
        0.0
    } else {
        hits as f64 / total_requests as f64 * 100.0
    };

    let watermark_bytes = max_bytes.saturating_mul(cache_watermark_percent()) / 100;
    if total_before > watermark_bytes {
        warn!(
            "缓存占用 {} 字节，超过水位线 {} 字节（配额 {} 字节的 {}%）",
            total_before,
            watermark_bytes,
            max_bytes,
            cache_watermark_percent()
        );
    }

    if total_before <= target_bytes {
        info!(
            "cache stats: entries={}, bytes={}, hit_rate={:.1}% (hits={}, misses={})",
            entries.len(),
            total_before,
            hit_rate,
            hits,
            misses
        );
        return;
    }

    let active_before = SystemTime::now().checked_sub(Duration::from_secs(ACTIVE_MINUTES * 60));
    let Some(active_before) = active_before else {
        return;
    };
    let evictions = select_evictions(entries, total_before, target_bytes, &protected, active_before);
    if evictions.is_empty() {
        warn!(
            "缓存占用 {} 字节超配额，但没有可安全淘汰的文件（受保护 {} 项）",
            total_before,
            protected.len()
        );
        return;
    }
    let (freed, count) = evict_files(&evictions);
    info!(
        "cache evicted: files={}, freed={} bytes, before={} bytes, hit_rate={:.1}% (hits={}, misses={}), in_flight_uploads={}",
        count,
        freed,
        total_before,
        hit_rate,
        hits,
        misses,
        vfs::IN_FLIGHT_WRITES.load(Ordering::SeqCst)
    );
}

#[cfg(test)]
mod tests;
