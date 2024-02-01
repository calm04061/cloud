use actix_web::web;
use api::{Capacity, Plugin};
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
#[no_mangle]
pub fn config(cfg: &mut web::ServiceConfig) {
    let generated = generate();
    cfg.service(actix_web_static_files::ResourceFiles::new("/", generated).resolve_not_found_to_root());
}
struct CloudUi;

impl Plugin for CloudUi {
    fn name(&self) -> &'static str {
        "cloud-ui"
    }

    fn version(&self) -> &'static str {
        "0.0.1"
    }

    fn capacities(&self) -> Vec<Capacity> {
        vec![Capacity::WEB("config".to_string())]
    }
}
#[no_mangle]
pub fn plugin_meta() -> Box<dyn Plugin> {
    Box::new(CloudUi)
}