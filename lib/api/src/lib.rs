use crate::error::ErrorInfo;
use libloading::Library;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod error;
pub mod util;

pub type ResponseResult<T> = Result<T, ErrorInfo>;

pub const ROOT_FILE_ID: u64 = 1;
pub const CLOUD_FILE_ROOT: &str = "CLOUD_FILE_ROOT";
pub const PLUGIN_ABI_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone)]
pub enum Capacity {
    WEB(String),
    STORAGE(String),
}

#[derive(Clone)]
pub enum PluginInstance {
    Native(Arc<Library>),
    Wasi(Arc<dyn WasiPlugin>),
}

pub struct PluginMetaInfo {
    pub meta_info: MetaInfo,
    pub instance: PluginInstance,
}

pub trait Plugin {
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn capacities(&self) -> Vec<Capacity>;
}

pub trait WasiPlugin: Send + Sync {
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn capacities(&self) -> Vec<Capacity>;
    fn call(&self, method: &str, params: &str) -> Result<String, String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetaInfo {
    pub name: String,
    pub version: String,
    pub capacities: Vec<Capacity>,
}
