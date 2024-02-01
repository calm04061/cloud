use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ErrorMessage {
    pub(crate) code: Option<String>,
    pub(crate) message: Option<String>,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct FileInfo {
    pub(crate) drive_id: Option<String>,
    pub(crate) file_id: Option<String>,
    pub(crate) parent_file_id: Option<String>,
    pub(crate) name: Option<String>,
}

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
    pub(crate) root_file_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct AuthToken {
    pub(crate) token_type: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: i32,
}