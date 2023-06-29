use actix_web::web;
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn config(cfg: &mut web::ServiceConfig) {
    let generated = generate();
    cfg.service(actix_web_static_files::ResourceFiles::new("/", generated).resolve_not_found_to_root());
}
