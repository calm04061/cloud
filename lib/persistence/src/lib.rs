use rbatis::rbdc::datetime::DateTime;
use strum_macros::EnumIter;

pub mod mapper;
pub mod support;
pub mod service;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileMeta {
    pub id: Option<i32>,
    pub name: String,
    pub parent_id: i32,
    pub file_type: i8,
    pub file_length: usize,
    pub status: i8,
    pub deleted: i8,
    pub create_time: i64,
    pub update_time: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileBlockMeta {
    pub id: Option<i32>,
    pub block_index: i64,
    pub file_part_id: String,
    pub update_time: i64,
    pub file_modify_time: i64,
    pub deleted: i8,
    pub file_meta_id: i32,
    pub part_hash: String,
    pub status: i8,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CloudFileBlock {
    pub id: Option<i32>,
    pub file_block_id: i32,
    pub cloud_meta_id: i32,
    pub cloud_file_id: Option<String>,
    pub cloud_file_hash: Option<String>,
    pub status: i8,
    pub deleted: i8,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

#[derive(Clone, Copy, Debug)]
pub enum FileStatus {
    Init,
    Uploading,
    UploadSuccess,
    UploadFail,
    FileNotExist,
    FileReadError,
    WaitClean,
    Cleaning,
    Cleaned,
    CleanFail,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub(crate) id: Option<i32>,
    pub(crate) property: String,
    pub value: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventMessage {
    pub id: Option<i32>,
    pub event_type: i8,
    pub event_result: i8,
    pub message: String,
    pub create_time: DateTime,
}

pub enum EventType {
    UploadFileBlock
}

pub enum EventResult {
    Success,
    Fail,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum FileMetaType {
    FILE,
    DIR,
    SYMLINK,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, EnumIter)]
pub enum CloudType {
    AliYun,
    Baidu,
    Local,
    OneDrive,
    Sftp,
}

#[derive(PartialEq, Debug, Clone)]
pub enum MetaStatus {
    WaitInit,
    WaitDataRoot,
    Enable,
    InvalidRefresh,
    Disabled,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CloudMeta {
    pub id: Option<i32>,
    pub name: String,
    pub auth: Option<String>,
    pub last_work_time: Option<i64>,
    pub data_root: Option<String>,
    pub status: i8,
    pub deleted: i8,
    pub cloud_type: i8,
    pub total_quota: Option<u64>,
    pub used_quota: Option<u64>,
    pub remaining_quota: Option<u64>,
    pub extra: Option<String>,
    pub expires_in: Option<u32>,
}
/// Config
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
}