use super::*;
use std::collections::HashSet;
use std::time::{Duration, SystemTime};

fn entry(name: &str, size: u64, modified: SystemTime) -> CacheEntry {
    CacheEntry {
        path: std::path::PathBuf::from(format!("/tmp/cache-test/{name}")),
        size,
        modified,
    }
}

fn protected_names(names: &[&str]) -> HashSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

fn minutes_before_now(minutes: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(minutes * 60)
}

/// 总量未超配额时不淘汰任何文件。
#[test]
fn no_eviction_when_under_quota() {
    let now = SystemTime::now();
    let entries = vec![entry("a", 100, now), entry("b", 100, now)];
    let plan = select_evictions(entries, 200, 400, &protected_names(&[]), now);
    assert!(plan.is_empty());
}

/// 按 mtime 从旧到新淘汰，受保护块与活跃文件不参与。
#[test]
fn evicts_oldest_unprotected_first() {
    let old = minutes_before_now(60);
    let recent = minutes_before_now(1);
    let now = SystemTime::now();

    let entries = vec![
        entry("protected-old", 100, old),   // 受保护：不淘汰
        entry("active", 100, recent),       // 活跃（10 分钟内）：不淘汰
        entry("oldest", 100, old),          // 最旧且不受保护：先淘汰
        entry("older", 100, minutes_before_now(30)),
    ];
    let plan = select_evictions(entries, 400, 200, &protected_names(&["protected-old"]), now);

    let names: Vec<String> = plan
        .iter()
        .filter_map(|e| e.path.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    assert_eq!(names, vec!["oldest".to_string(), "older".to_string()]);
}

/// 淘汰足够数量后即停止（降到目标水位以内），不会清空全部。
#[test]
fn stops_when_target_reached() {
    let old = minutes_before_now(60);
    let now = SystemTime::now();

    let entries = vec![
        entry("a", 100, old),
        entry("b", 100, old),
        entry("c", 100, old),
        entry("d", 100, old),
    ];
    // 总量 400，目标 250 → 淘汰 2 个即可
    let plan = select_evictions(entries, 400, 250, &protected_names(&[]), now);
    assert_eq!(plan.len(), 2);
}

/// 受保护块占满候选时宁可超配额也不淘汰。
#[test]
fn keeps_protected_when_no_candidates() {
    let old = minutes_before_now(60);
    let now = SystemTime::now();

    let entries = vec![entry("p1", 100, old), entry("p2", 100, old)];
    let plan = select_evictions(entries, 200, 50, &protected_names(&["p1", "p2"]), now);
    assert!(plan.is_empty());
}
