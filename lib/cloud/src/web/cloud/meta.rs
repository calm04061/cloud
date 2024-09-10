use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, Responder, Result};

use persistence::meta::CloudMeta;
use persistence::CloudType::Local;
#[cfg(not(windows))]
use persistence::CloudType::Sftp;
use persistence::{CloudType, MetaStatus};
use service::meta::CloudMetaManager;
use service::CONTEXT;

use crate::web::common::WebResult;
use crate::web::vo::cloud::CloudMetaVo;

#[get("/storage/meta")]
pub(crate) async fn list() -> impl Responder {
    let meta = CONTEXT.cloud_meta_manager.list().await.unwrap();
    let x = &Some(meta);
    WebResult::actix_web_json_result(x)
}

#[post("/storage/meta")]
pub(crate) async fn new(
    meta: Json<CloudMetaVo>,
) -> impl Responder {
    let vo = meta.0.clone();
    let mut meta = CloudMeta::from(vo);
    let cloud_type :CloudType = meta.cloud_type.into();
    match cloud_type {
        Local => {
            meta.status = MetaStatus::Enable.into();
        }
		#[cfg(not(windows))]
        Sftp => {
            meta.status = MetaStatus::Enable.into();
        }
        _ => {}
    }
    let meta = CONTEXT.cloud_meta_manager.add(&meta).await.unwrap();
    WebResult::actix_web_json_result(&meta.id)
}

#[get("/storage/meta/{id}")]
pub(crate) async fn info(
    id: Path<i32>,
) -> Result<impl Responder> {
    let x = CONTEXT.cloud_meta_manager.info(id.into_inner()).await?;
    Ok(WebResult::actix_web_json_result(&Some(x)))
}

#[post("/storage/meta/{id}")]
pub(crate) async fn update(
    id: Path<i32>,
    meta: Json<CloudMetaVo>,
) -> Result<impl Responder> {
    let mut meta_db = CONTEXT.cloud_meta_manager.info(id.into_inner()).await?;
    meta_db.name = meta.name.clone();
    meta_db.auth = Option::from(meta.auth.clone().unwrap_or("{}".to_string()));
    meta_db.data_root = meta.data_root.clone();
    let x = CONTEXT.cloud_meta_manager.update_meta(&meta_db).await?;
    Ok(WebResult::actix_web_json_result(&Some(x)))
}

#[delete("/storage/meta/{id}")]
pub(crate) async fn delete(
    id: Path<i32>,
) -> Result<impl Responder> {
    let x = CONTEXT.cloud_meta_manager.delete(id.into_inner()).await?;
    Ok(WebResult::actix_web_json_result(&Some(x)))
}

#[post("/storage/meta/{id}/enable")]
pub(crate) async fn enable(
    id: Path<i32>,
) -> Result<impl Responder> {
    let meta = CONTEXT.cloud_meta_manager.info(id.into_inner()).await?;
    let mut meta = meta.clone();
    meta.status = MetaStatus::Enable.into();
    let x = CONTEXT.cloud_meta_manager.update_meta(&meta).await?;
    Ok(WebResult::actix_web_json_result(&Some(x)))
}

#[post("/storage/meta/{id}/disable")]
pub(crate) async fn disable(
    id: Path<i32>,
) -> Result<impl Responder> {
    let meta = CONTEXT.cloud_meta_manager.info(id.into_inner()).await?;
    let mut meta = meta.clone();
    meta.status = MetaStatus::Disabled.into();
    let x = CONTEXT.cloud_meta_manager.update_meta(&meta).await?;
    Ok(WebResult::actix_web_json_result(&Some(x)))
}