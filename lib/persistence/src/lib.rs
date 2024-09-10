use strum_macros::EnumIter;
use chrono::{DateTime, Utc};

pub mod support;
pub mod service;
pub mod meta {
    use chrono::{DateTime, Utc};

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
    pub struct FileMeta {
        pub id: Option<i64>,
        pub name: String,
        pub parent_id: i64,
        pub file_type: i8,
        pub mode: i32,
        pub gid: i32,
        pub uid: i32,
        pub file_length: i64,
        pub status: i8,
        pub deleted: i8,
        pub create_time: DateTime<Utc>,
        pub update_time: DateTime<Utc>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
    pub struct FileBlockMeta {
        pub id: Option<i32>,
        pub block_index: i64,
        pub file_part_id: String,
        pub update_time: DateTime<Utc>,
        pub file_modify_time: i64,
        pub deleted: i8,
        pub file_meta_id: i64,
        pub part_hash: String,
        pub status: i8,
    }
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
    pub struct CloudMeta {
        pub id: Option<i32>,
        pub name: String,
        pub auth: Option<String>,
        pub last_work_time: Option<i64>,
        pub data_root: Option<String>,
        pub status: i8,
        pub deleted: i8,
        pub cloud_type: i8,
        pub total_quota: Option<i64>,
        pub used_quota: Option<i64>,
        pub remaining_quota: Option<i64>,
        pub extra: Option<String>,
        pub expires_in: Option<i64>,
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct CloudFileBlock {
    pub id: Option<i32>,
    pub file_block_id: i32,
    pub cloud_meta_id: i32,
    pub cloud_file_id: Option<String>,
    pub cloud_file_hash: Option<String>,
    pub status: i8,
    pub deleted: i8,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Config {
    pub id: Option<i32>,
    pub property: String,
    pub value: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EventMessage {
    pub id: Option<i32>,
    pub event_type: i8,
    pub event_result: i8,
    pub message: String,
    pub create_time: DateTime<Utc>,
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
    #[cfg(not(windows))]
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


/// Config
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub password: String,
}