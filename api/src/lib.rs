use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub enum Capacity{
    WEB(String)
}
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn capacities(&self) ->Vec<Capacity>;
}
#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    pub name: String,
    pub version: String,
    pub capacities: Vec<Capacity>,
}
