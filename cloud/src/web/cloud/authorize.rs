use crate::web::vo::auth::Callback;
use crate::web::AppState;
use actix_web::web::{Data, Path, Query};
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use actix_web::http::header::HeaderMap;

#[get("/authorize/storage/{id}")]
pub(crate) async fn authorize(
    id: Path<i32>,
    state: Data<AppState>,
    request: HttpRequest,
) -> impl Responder {
    let headers = request.headers();
    let server = header_value(headers, "Host", "pan.calm0406.com:8080");

    let mut guard = state.facade_cloud.lock().unwrap();
    let url = guard.authorize(&server, id.into_inner()).await;
    HttpResponse::MovedPermanently()
        .append_header(("Location", url.as_str()))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

#[get("/cloud/callback")]
pub(crate) async fn callback(state: Query<Callback>, app_state: Data<AppState>, request: HttpRequest) -> impl Responder {
    let data = app_state.clone();
    let headers = request.headers();
    let server = header_value(headers, "Host", "pan.calm0406.com:8080");

    let mut guard = data.facade_cloud.lock().unwrap();
    let url = format!("/cloud?callback=true");
    guard.callback(&server, &state.into_inner()).await;
    HttpResponse::MovedPermanently()
        .append_header(("Location", url.as_str()))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

fn header_value(headers: &HeaderMap, header: &str, default_value: &str) -> String {
    let option = headers.get(header);
    let server = match option {
        None => {
            default_value
        }
        Some(value) => {
            value.to_str().unwrap()
        }
    };
    server.to_string()

}
