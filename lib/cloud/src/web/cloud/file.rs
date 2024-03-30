use actix_web::web::Json;
use actix_web::{delete, get, web, Responder, Result};

use service::meta::FileManager;
use service::CONTEXT;

use crate::web::common::WebResult;

#[get("/file/{id}")]
pub(crate) async fn info(id: web::Path<u64>) -> Result<impl Responder> {
    let x = CONTEXT.file_manager.info_by_id(id.into_inner()).await;
    let x1 = x.unwrap();
    Ok(WebResult::actix_web_json_result(&Some(x1)))
}
#[delete("/file/{id}")]
pub(crate) async fn delete(id: web::Path<u64>) -> Result<impl Responder> {
    let meta = CONTEXT.file_manager.delete_file_meta(id.into_inner()).await?;
    Ok(WebResult::actix_web_json_result(&Some(meta)))
}

#[get("/files/{file_id}")]
pub(crate) async fn files(file_id: web::Path<u64>) -> Result<impl Responder> {
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
