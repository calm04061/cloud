use actix_web::web;

pub(crate) mod authorize;
mod file;
#[cfg(not(windows))]
pub(crate) mod fs;
pub(crate) mod meta;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(meta::list);
    cfg.service(meta::new);
    cfg.service(meta::info);
    cfg.service(meta::update);
    cfg.service(meta::delete);
    cfg.service(file::info);
    cfg.service(file::delete);
    cfg.service(file::files);
    cfg.service(file::root_files);
    cfg.service(authorize::authorize);
    cfg.service(authorize::callback);
    #[cfg(not(windows))]
    fs(cfg);
}

#[cfg(not(windows))]
fn fs(cfg: &mut web::ServiceConfig) {
    use crate::web::cloud::fs::FsManager;
    use actix_web::web::Data;
    let manager = FsManager::new();

    cfg.app_data(Data::new(manager))
        .service(fs::umount)
        .service(fs::mount);
}
