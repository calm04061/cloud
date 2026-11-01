use actix_web::http::header::HeaderMap;
use actix_web::web::{Path, Query};
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use log::error;
use storage::model::AuthMethod;

use storage::web::auth::Callback;
use storage::storage_facade;

#[get("/authorize/storage/{id}")]
pub(crate) async fn authorize(
    id: Path<i32>,
    request: HttpRequest,
) -> impl Responder {
    let server = server_base_url(request.headers());
    let id = id.into_inner();
    let vec = storage_facade().get_auth_methods(id).await;
    let mut target = String::from("/cloud");
    if vec.contains(&AuthMethod::OAuth2) {
        target = match storage_facade().authorize(&server, id).await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to authorize: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
    }
    HttpResponse::Found()
        .append_header(("Location", target))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

#[get("/cloud/callback")]
pub(crate) async fn callback(state: Query<Callback>, request: HttpRequest) -> impl Responder {
    let server = server_base_url(request.headers());

    let url = "/cloud".to_string();
    let result = storage_facade().callback(&server, &state.into_inner()).await;
    if let Err(e) = result {
        error!("{:?}", e);
    }
    HttpResponse::Found()
        .append_header(("Location", url.as_str()))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

fn header_value(headers: &HeaderMap, header: &str, default_value: &str) -> String {
    let option = headers.get(header);
    let server = match option {
        None => default_value,
        Some(value) => match value.to_str() {
            Ok(s) => s,
            Err(_) => default_value,
        },
    };
    server.to_string()
}

fn server_base_url(headers: &HeaderMap) -> String {
    if let Ok(base_url) = dotenvy::var("PUBLIC_BASE_URL") {
        let base_url = base_url.trim().trim_end_matches('/');
        if !base_url.is_empty() {
            return base_url.to_string();
        }
    }
    format!("http://{}", header_value(headers, "Host", "127.0.0.1:8088"))
}
