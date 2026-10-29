use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WebCapacity {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct StorageCapacity {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Capacity {
    WEB(WebCapacity),
    STORAGE(StorageCapacity),
}

const PLUGIN_NAME: &str = "demo-wasi-plugin";
const PLUGIN_VERSION: &str = "1.0.0";
const CAPACITIES_JSON: &str =
    r#"[{"type":"WEB","path":"/demo"},{"type":"STORAGE","name":"demo_storage"}]"#;

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
    CAPACITIES_JSON.as_ptr()
}

#[no_mangle]
pub fn alloc(ptr: i32, len: i32) -> i32 {
    ptr
}

#[no_mangle]
pub fn call_hello(ptr: i32, len: i32) -> i32 {
    let input = unsafe {
        let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
        std::str::from_utf8_unchecked(slice)
    };

    let response = format!("Hello from WASI plugin! Input: {}", input);
    let response_bytes = response.as_bytes();

    unsafe {
        let dst = std::slice::from_raw_parts_mut(ptr as *mut u8, response_bytes.len());
        dst.copy_from_slice(response_bytes);
    }

    response_bytes.len() as i32
}

#[no_mangle]
pub fn call_echo(ptr: i32, len: i32) -> i32 {
    ptr
}
