use actix_web::web;
use dav_server::actix::{DavRequest, DavResponse};
use dav_server::fakels::FakeLs;
use dav_server::{DavConfig, DavHandler};

use crate::fs::dav::cluod_dav_filesystem::{CloudDavFs, DAV_PREFIX};
use crate::fs::vfs::DEFAULT_TEMP_PATH;

pub async fn dav_handler(req: DavRequest, dav_handler: web::Data<DavHandler>) -> DavResponse {
    if let Some(prefix) = req.prefix() {
        let config = DavConfig::new().strip_prefix(prefix);
        dav_handler.handle_with(config, req.request).await.into()
    } else {
        dav_handler.handle(req.request).await.into()
    }
}

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    let cache_file = String::from(DEFAULT_TEMP_PATH);
    let dav_fs = CloudDavFs::new(cache_file.as_str(), DAV_PREFIX);
    let dav_server = DavHandler::builder()
        .filesystem(Box::new(dav_fs))
        .locksystem(FakeLs::new())
        .autoindex(true)
        .build_handler();

    cfg.app_data(web::Data::new(dav_server.clone()))
        .service(web::resource("{tail:.*}").to(dav_handler));
}