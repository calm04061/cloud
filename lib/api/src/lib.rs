pub mod error;
pub mod util;

use serde::{Deserialize, Serialize};
use crate::error::ErrorInfo;

pub type ResponseResult<T> = Result<T, ErrorInfo>;

#[derive(Serialize, Deserialize)]
pub enum Capacity {
    WEB(String)
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
