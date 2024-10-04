use crate::error::ErrorInfo;
use libloading::Library;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod util;

pub type ResponseResult<T> = Result<T, ErrorInfo>;

pub const ROOT_FILE_ID: u64 = 1;
pub const CLOUD_FILE_ROOT: &str = "CLOUD_FILE_ROOT";

#[derive(Serialize, Deserialize)]
pub enum Capacity {
    WEB(String),
    STORAGE(String),
}

pub struct PluginMetaInfo {
    pub meta_info: MetaInfo,
    pub library: Library,
}

pub trait Plugin {
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn capacities(&self) -> Vec<Capacity>;
}

#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    pub name: String,
    pub version: String,
    pub capacities: Vec<Capacity>,
}
