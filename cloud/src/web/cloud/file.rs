use actix_web::{delete, get, Responder, Result, web};
use actix_web::web::Json;

use crate::database::meta::FileManager;
use crate::service::CONTEXT;
use crate::web::common::WebResult;

#[get("/file/{id}")]
pub(crate) async fn info(id: web::Path<i32>) -> Result<impl Responder> {
    let x = CONTEXT.file_manager.info_by_id(id.into_inner()).await;
    let option = x.as_ref();
    let x1 = option.unwrap();
    Ok(WebResult::actix_web_json_result(x1))
}
#[delete("/file/{id}")]
pub(crate) async fn delete(id: web::Path<i32>) -> Result<impl Responder> {
    let x = CONTEXT.file_manager.delete_file_meta(id.into_inner()).await;
    Ok(WebResult::actix_web_json_result(&x))
}

#[get("/files/{file_id}")]
pub(crate) async fn files(file_id: web::Path<i32>) -> Result<impl Responder> {
    let x = CONTEXT
        .file_manager
        .list_by_parent(file_id.into_inner())
        .await;
    Ok(Json(WebResult::success(x.clone())))
}

#[get("/files")]
pub(crate) async fn root_files() -> Result<impl Responder> {
    let x = CONTEXT.file_manager.list_by_parent(1).await;
    Ok(Json(WebResult::success(x.clone())))
}
