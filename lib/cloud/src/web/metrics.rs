use actix_web::{get, HttpResponse, Responder};
use std::sync::atomic::Ordering;

use crate::fs::vfs;
use service::CONTEXT;

/// Prometheus 文本格式指标端点（text/plain; version=0.0.4）。
///
/// 暴露：本地缓存命中/未命中、在途 VFS 写入数、上传队列按状态分布。
/// 无认证，仅供内网抓取；如需隔离请用反代做访问控制。
#[get("/metrics")]
pub(crate) async fn metrics() -> impl Responder {
    let mut lines: Vec<String> = Vec::new();

    let (hits, misses) = vfs::cache_stats();
    lines.push("# TYPE cloud_cache_hits counter".to_string());
    lines.push(format!("cloud_cache_hits {}", hits));
    lines.push("# TYPE cloud_cache_misses counter".to_string());
    lines.push(format!("cloud_cache_misses {}", misses));

    lines.push("# TYPE cloud_vfs_in_flight_writes gauge".to_string());
    lines.push(format!(
        "cloud_vfs_in_flight_writes {}",
        vfs::IN_FLIGHT_WRITES.load(Ordering::SeqCst)
    ));

    lines.push("# TYPE cloud_upload_queue gauge".to_string());
    match CONTEXT.cloud_file_block_manager.status_counts().await {
        Ok(counts) => {
            for (status, count) in counts {
                lines.push(format!("cloud_upload_queue{{status=\"{}\"}} {}", status, count));
            }
        }
        Err(e) => {
            log::error!("metrics: query upload queue failed: {}", e);
            return HttpResponse::ServiceUnavailable()
                .content_type("text/plain; version=0.0.4")
                .body(lines.join("\n"));
        }
    }

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(lines.join("\n"))
}
