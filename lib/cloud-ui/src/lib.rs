use std::env;

use actix_web::web;

use api::{Capacity, Plugin};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

#[no_mangle]
pub fn config(cfg: &mut web::ServiceConfig) {
    let generated = generate();
    cfg.service(actix_web_static_files::ResourceFiles::new("/", generated).resolve_not_found_to_root());
}
struct CloudUi;

impl Plugin for CloudUi {
    fn name(&self) -> String {
        "cloud-ui".to_string()
    }

    fn version(&self) -> String {
        // let version = "0.0.1".to_string();
        VERSION.to_string()
    }

    fn capacities(&self) -> Vec<Capacity> {
        vec![Capacity::WEB("config".to_string())]
    }
}
#[no_mangle]
pub fn plugin_meta() -> Box<dyn Plugin> {
    Box::new(CloudUi)
}