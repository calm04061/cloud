use actix_web::{get, HttpResponse, Responder};
use serde_derive::Serialize;
use service::CONTEXT;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
}

/// 存活探针：进程在即返回 200，不依赖任何外部资源。
#[get("/health")]
pub(crate) async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthStatus { status: "UP" })
}

/// 就绪探针：数据库可访问才返回 200。
#[get("/ready")]
pub(crate) async fn ready() -> impl Responder {
    match CONTEXT.config_manager.info(api::CLOUD_FILE_ROOT).await {
        Some(_) => HttpResponse::Ok().json(HealthStatus { status: "READY" }),
        None => {
            log::error!("readiness check failed: database unreachable");
            HttpResponse::ServiceUnavailable().json(HealthStatus { status: "NOT_READY" })
        }
    }
}
