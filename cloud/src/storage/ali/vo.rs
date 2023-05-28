use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DriveCapacity {
    pub(crate) total_size: u64,
    pub(crate) used_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DevicePersonalInfo {
    // personal_rights_info:HashMap<>
    pub(crate) personal_space_info: DriveCapacity,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ErrorResult {
    pub(crate) code: String,
    message: String,
    #[serde(rename(serialize = "requestId", deserialize = "requestId"))]
    request_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DriveInfo {
    pub(crate) default_drive_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AliExtra {
    pub(crate) drive_id: Option<String>,
}
