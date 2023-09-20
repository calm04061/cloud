use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct VisualProUserInfo {
    pub(crate) msisdn: Option<String>,
    pub(crate) user_id: Option<String>,
    area_code: String,
    qry_info_list: Vec<QryInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct QryInfo {
    qry_info: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChinaMobileResult<T> {
    pub(crate) code: i32,
    pub(crate) message: Option<String>,
    pub(crate) data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AccessToken {
    pub(crate) access_token: String,
    pub(crate) expires_in: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(crate) struct DiskInfo {
    pub(crate) free_disk_size: u64,
    pub(crate) disk_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PcUploadFileRequest {
    pub(crate) total_size: u64,
    pub(crate) upload_content_list: Vec<UploadContentInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UploadContentInfo {
    pub(crate) content_name: String,
    pub(crate) content_size: u64,
    pub(crate) content_desc: String,
    pub(crate) content_tag_list: String,
    pub(crate) comlex_flag: i32,
    pub(crate) comlex_cid: String,
    pub(crate) res_cid: String,
    pub(crate) digest: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct UploadResult {
    pub(crate) uploadTaskID: String,
    pub(crate) redirectionUrl: String,
    pub(crate) newContentIDList: Vec<NewContent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct NewContent {
    pub(crate) contentID: String,
    pub(crate) contentName: String,
    needUpload: i8,
    fileEtag: String,
    fileVersion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GetContentInfo {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ContentInfo {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DelCatalogContent {
    pub(crate) content_id: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DelContentCatalogRes {}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DownloadRequest {
    contentID: String,
    OwnerMSISDN: String,
    entryShareCatalogID: String,
}
