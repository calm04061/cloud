use serde::{Deserialize, Serialize};

#[derive(PartialEq)]
pub enum AuthMethod {
    OAuth2,
    UsernamePassword,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileItemWrapper {
    next_marker: String,
    punished_file_count: i64,
    items: Vec<FileItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ImageMediaMetadata {
    width: i32,
    height: i32,
    image_tags: Vec<ImageTag>,
    image_quality: ImageQuality,
    cropping_suggestion: Vec<CroppingSuggestion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quota {
    pub total: i64,
    pub used: i64,
    pub remaining: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct CroppingSuggestion {
    aspect_ratio: String,
    score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageQuality {
    overall_score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageTag {
    confidence: f64,
    name: String,
    tag_level: i32,
    centric_score: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ExFieldsInfo {
    image_count: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateResponse {
    pub encrypt_mode: String,
    pub file_id: String,
    pub file_name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub file_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileItem {
    create_at: Option<String>,
    creator_id: Option<String>,
    creator_name: Option<String>,
    creator_type: Option<String>,
    encrypt_mode: String,
    file_id: String,
    hidden: bool,
    last_modifier_id: Option<String>,
    last_modifier_name: Option<String>,
    last_modifier_type: Option<String>,
    name: String,
    revision_id: String,
    starred: bool,
    status: String,
    trashed: Option<bool>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    file_type: String,
    updated_at: String,
    user_meta: Option<String>,
    labels: Option<Vec<String>>,
    upload_id: Option<String>,
    parent_file_id: Option<String>,
    crc64_hash: Option<String>,
    content_hash: Option<String>,
    content_hash_name: Option<String>,
    download_url: Option<String>,
    url: Option<String>,
    thumbnail: Option<String>,
    image_media_metadata: Option<ImageMediaMetadata>,
    category: Option<String>,
    punish_flag: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    items: Vec<FileItem>,
    next_marker: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub(crate) create_at: Option<String>,
    pub(crate) creator_id: Option<String>,
    pub(crate) creator_name: Option<String>,
    pub(crate) creator_type: Option<String>,
    pub(crate) domain_id: Option<String>,
    pub(crate) drive_id: Option<String>,
    pub(crate) encrypt_mode: Option<String>,
    pub(crate) ex_fields_info: Option<ExFieldsInfo>,
    pub file_id: String,
    pub(crate) path: Option<String>,
    pub(crate) hidden: Option<bool>,
    pub(crate) last_modifier_id: Option<String>,
    pub(crate) last_modifier_name: Option<String>,
    pub(crate) last_modifier_type: Option<String>,
    pub(crate) name: String,
    pub(crate) revision_id: Option<String>,
    pub(crate) starred: bool,
    pub(crate) status: Option<String>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) updated_at: Option<String>,
    pub(crate) user_meta: Option<String>,

    pub(crate) labels: Option<Vec<String>>,
    pub(crate) upload_id: Option<String>,
    pub(crate) parent_file_id: Option<String>,
    pub(crate) crc64_hash: Option<String>,
    pub(crate) content_hash: Option<String>,
    pub(crate) content_hash_name: Option<String>,
    pub(crate) download_url: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) thumbnail: Option<String>,
    pub(crate) image_media_metadata: Option<ImageMediaMetadata>,
    pub(crate) category: Option<String>,
    pub(crate) punish_flag: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub(crate) domain_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) avatar: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) nick_name: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) user_name: Option<String>,
    pub(crate) default_drive_id: Option<String>,
    // "user_data": {},
    pub(crate) deny_change_password_by_self: Option<bool>,
    pub(crate) need_change_password_next_login: Option<bool>,
    // "permission": null
    pub(crate) creator: Option<String>,

    pub(crate) created_at: Option<i64>,
    pub(crate) updated_at: Option<i64>,
}