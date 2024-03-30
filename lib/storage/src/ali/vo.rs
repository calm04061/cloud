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
pub(crate) struct AliAuthToken {
    pub(crate) token_type: String,
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DownloadUrl {
    expiration: String,
    method: String,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DriveFile {
    pub(crate) drive_id: String,
    pub(crate) file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PartInfo {
    pub(crate) part_number: u32,
    pub(crate) upload_url: Option<String>,
    pub(crate) internal_upload_url: Option<String>,
    pub(crate) content_type: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    ding_ding_robot_url: String,
    encourage_desc: String,
    feed_back_switch: bool,
    following_desc: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CreateFile {
    pub(crate) drive_id: String,
    pub(crate) parent_file_id: String,
    pub(crate) part_info_list: Vec<PartInfo>,
    pub(crate) name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) check_name_mode: Option<String>,
    pub(crate) size: u64,
    pub(crate) pre_hash: String,
    pub(crate) content_hash: Option<String>,
    pub(crate) content_hash_name: Option<String>,
    pub(crate) proof_code: Option<String>,
    pub(crate) proof_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CompleteRequest {
    // {"drive_id":"2050438","upload_id":"617283DD041046B0A97AA79857DDDBBE","file_id":"621cf518a5ddef2ebc7647519486ec82de248fe0"}
    pub(crate) drive_id: String,
    pub(crate) file_id: String,
    pub(crate) upload_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UploadPreResult {
    parent_file_id: String,
    pub(crate) upload_id: Option<String>,
    rapid_upload: bool,
    pub(crate) exist: Option<bool>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) file_id: String,
    domain_id: String,
    drive_id: String,
    pub(crate) encrypt_mode: String,
    pub(crate) file_name: String,
    pub(crate) part_info_list: Option<Vec<PartInfo>>,
}