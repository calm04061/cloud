use actix_web::http::header::HeaderMap;
use actix_web::web::{Path, Query};
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use log::error;
use storage::model::AuthMethod;

use storage::web::auth::Callback;
use storage::STORAGE_FACADE;

#[get("/authorize/storage/{id}")]
pub(crate) async fn authorize(
    id: Path<i32>,
    request: HttpRequest,
) -> impl Responder {
    let headers = request.headers();
    let server = header_value(headers, "Host", "pan.calm0406.com:8080");
    let server = format!("http://{server}");
    let id = id.into_inner();
    let mut cloud = STORAGE_FACADE.write().await;
    let vec = cloud.get_auth_methods(id).await;
    let mut target = String::from("/cloud");
    if vec.contains(&AuthMethod::OAuth2) {
        target = match cloud.authorize(&server, id).await {
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
    let headers = request.headers();
    let server = header_value(headers, "Host", "pan.calm0406.com:8080");

    let url = "/cloud".to_string();
    let result = STORAGE_FACADE.write().await.callback(&server, &state.into_inner()).await;
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
