use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Capacity {
    WEB(String),
    STORAGE(String),
}

static PLUGIN_NAME: &str = "test-wasi-plugin";
static PLUGIN_VERSION: &str = "0.1.0";
static CAPACITIES_JSON: &str = r#"[{"WEB":"test_web"},{"STORAGE":"test_storage"}]"#;

#[no_mangle]
pub fn name() -> *const u8 {
    let s = PLUGIN_NAME.as_bytes();
    s.as_ptr()
}

#[no_mangle]
pub fn version() -> *const u8 {
    let s = PLUGIN_VERSION.as_bytes();
    s.as_ptr()
}

#[no_mangle]
pub fn capacities() -> *const u8 {
    let s = CAPACITIES_JSON.as_bytes();
    s.as_ptr()
}

#[no_mangle]
pub fn alloc(_ptr: i32, _len: i32) -> i32 {
    0
}

#[no_mangle]
pub fn call_test(_param: i32) -> i32 {
    0
}
