use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WebConfig {
    pub path: String,
}

const PLUGIN_NAME: &str = "demo-wasi-web";
const PLUGIN_VERSION: &str = "1.0.0";
const WEB_CONFIG_JSON: &str = r#"[{"WEB":"/ui"},{"WEB":"/"}]"#;

#[no_mangle]
pub fn name() -> *const u8 {
    PLUGIN_NAME.as_ptr()
}

#[no_mangle]
pub fn version() -> *const u8 {
    PLUGIN_VERSION.as_ptr()
}

#[no_mangle]
pub fn capacities() -> *const u8 {
    WEB_CONFIG_JSON.as_ptr()
}

#[no_mangle]
pub fn web_config() -> *const u8 {
    WEB_CONFIG_JSON.as_ptr()
}

#[no_mangle]
pub fn alloc(_ptr: i32, _len: i32) -> i32 {
    0
}
