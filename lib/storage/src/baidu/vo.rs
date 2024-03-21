use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResult {
    // personal_rights_info:HashMap<>
    pub(crate) code: String,
    message: String,
    #[serde(rename(serialize = "requestId", deserialize = "requestId"))]
    request_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaiduUser {
    pub(crate) avatar_url: Option<String>,
    pub(crate) baidu_name: String,
    errmsg: String,
    errno: i32,
    netdisk_name: String,
    request_id: String,
    uk: i64,
    vip_type: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriveFile {
    pub(crate) drive_id: String,
    pub(crate) file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Query {
    pub(crate) drive_id: String,
    pub(crate) parent_file_id: String,
    pub(crate) limit: i32,
    pub(crate) all: bool,
    pub(crate) url_expire_sec: i32,
    pub(crate) image_thumbnail_process: String,
    pub(crate) image_url_process: String,
    pub(crate) video_thumbnail_process: String,
    pub(crate) fields: String,
    pub(crate) order_by: String,
    pub(crate) order_direction: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Search {
    pub(crate) drive_id: String,
    pub(crate) limit: i32,
    pub(crate) order_by: String,
    pub(crate) query: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct CompleteRequest {
    // {"drive_id":"2050438","upload_id":"617283DD041046B0A97AA79857DDDBBE","file_id":"621cf518a5ddef2ebc7647519486ec82de248fe0"}
    pub(crate) drive_id: String,
    pub(crate) file_id: String,
    pub(crate) upload_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DownloadUrl {
    content_hash: String,
    content_hash_name: String,
    crc64_hash: String,
    expiration: String,
    internal_url: String,
    method: String,
    size: u64,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BaiduPreCreate {
    pub(crate) uploadid: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) errmsg: Option<String>,
    pub(crate) errno: i32,
    return_type: Option<i32>,
    pub(crate) block_list: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BaiduCreate {
    pub(crate) category: Option<i32>,
    pub(crate) ctime: Option<i64>,
    from_type: Option<i32>,
    pub(crate) fs_id: Option<i64>,
    isdir: Option<i32>,
    md5: Option<String>,
    mtime: Option<i64>,
    path: Option<String>,
    server_filename: Option<String>,
    size: Option<i64>,
    errmsg: Option<String>,
    errno: i32,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BaiduFileMeta {
    category: i32,
    date_taken: Option<i64>,
    pub(crate) dlink: Option<String>,
    pub(crate) filename: String,
    pub(crate) fs_id: i64,
    isdir: i8,
    md5: String,
    oper_id: i64,
    pub(crate) path: String,
    server_ctime: i64,
    server_mtime: i64,
    size: i64,
    thumbs: Option<Thumb>,
    height: Option<i64>,
    width: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Thumb {
    icon: String,
    url1: String,
    url2: String,
    url3: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct FileMetas {
    errmsg: Option<String>,
    errno: i32,
    pub(crate) list: Vec<BaiduFileMeta>,
    // names: String,
    request_id: String,
}

pub(crate) enum BaiduOpera {
    Delete,
    // Copy,
    // Move,
    // Rename,
}

pub(crate) enum AsyncType {
    Async,
    // SelfAdaption,
    // Sync,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ManageFile {
    path: String,
    newname: Option<String>,
    dest: Option<String>,
    errno: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BaiduFileManagerResult {
    errmsg: Option<String>,
    errno: i32,
    request_id: i64,
    info: Vec<ManageFile>,
    taskid: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BaiduQuota {
    errno: i32,
    pub(crate) total: u64,
    free: u64,
    request_id: i64,
    expire: bool,
    pub(crate) used: u64,
}

//{"expires_in":2592000,"refresh_token":"122.6d27d0dac1f3e497a2c5ea18b9bb87be.YGOBz1S9vUHf0FjrFG-XFBS2lSxXVDQ9L5UZZUn.l1Imbw","access_token":"121.fc08dafbfbb8fe068a85bccae729cbc7.YgnZRGeB3OdpwwfnkWhUf3rS9WZ-o7ehMBH_5w-.V8c1Iw","session_secret":"","session_key":"","scope":"basic netdisk"}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Token {
    pub(crate) expires_in: u32,
    pub(crate) refresh_token: String,
    pub(crate) access_token: String,
}