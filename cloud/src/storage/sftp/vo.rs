use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HostUser {
    pub(crate) username: String,
    pub(crate) hostname: String,
    pub(crate) port: String,
    pub(crate) password: Option<String>,
    pub(crate) auth_type: Option<String>,
}