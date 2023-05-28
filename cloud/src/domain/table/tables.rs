use rbatis::rbdc::datetime::DateTime;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CloudFileBlock {
    pub id: Option<i64>,
    pub file_block_id: i64,
    pub cloud_meta_id: i32,
    pub cloud_file_id: Option<String>,
    pub cloud_file_hash: Option<String>,
    pub status: i8,
    pub deleted: i8,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileBlockMeta {
    pub(crate) id: Option<i64>,
    pub(crate) block_index: i64,
    pub(crate) file_part_id: String,
    // #[deprecated]
    // pub(crate) cloud_file_id: String,
    pub(crate) update_time: i64,
    pub(crate) file_modify_time: i64,
    pub(crate) deleted: i8,
    pub(crate) file_meta_id: i64,
    pub(crate) part_hash: Option<String>,
    // #[deprecated]
    // pub(crate) cloud_file_hash: Option<String>,
    pub(crate) status: i8,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileMeta {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) parent_id: i64,
    pub(crate) file_type: i8,
    pub(crate) file_length: usize,
    pub(crate) status: i8,
    pub(crate) deleted: i8,
    pub(crate) create_time: i64,
    pub(crate) update_time: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CloudMeta {
    pub(crate) id: Option<i32>,
    pub(crate) name: String,
    pub token: Option<String>,
    pub last_work_time: Option<i64>,
    pub data_root: Option<String>,
    pub status: i8,
    pub deleted: i8,
    pub cloud_type: i8,
    pub total_quota: Option<u64>,
    pub used_quota: Option<u64>,
    pub remaining_quota: Option<u64>,
    pub extra: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub(crate) property: String,
    pub(crate) value: String,
}
