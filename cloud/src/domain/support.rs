use crate::database::meta::FileStatus;
use crate::domain::table::tables::{CloudFileBlock, CloudMeta, Config, FileBlockMeta, FileMeta};
use rbatis::rbdc::datetime::{DateTime};

impl CloudFileBlock {
    pub(crate) fn init(file_block_meta_id: i32, cloud_meta_id: i32) -> Self {
        CloudFileBlock {
            id: Some(0),
            file_block_id: file_block_meta_id,
            cloud_meta_id,
            cloud_file_id: None,
            cloud_file_hash: None,
            status: FileStatus::Init.into(),
            deleted: 0,
            create_time: DateTime::now(),
            update_time: DateTime::now(),
        }
    }
}

impl Default for CloudMeta {
    fn default() -> Self {
        CloudMeta{
            id: Some(0),
            name: "".to_string(),
            token: None,
            last_work_time: None,
            data_root: None,
            status: 0,
            deleted: 0,
            cloud_type: 0,
            total_quota: None,
            used_quota: None,
            remaining_quota: None,
            extra: None,
            expires_in: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config{
            id: Some(0),
            property: "".to_string(),
            value: "".to_string(),
        }
    }
}

impl Default for FileMeta {
    fn default() -> Self {
        FileMeta{
            id: Some(0),
            name: "".to_string(),
            parent_id: 0,
            file_type: 0,
            file_length: 0,
            status: 0,
            deleted: 0,
            create_time: 0,
            update_time: 0,
        }
    }
}

impl Default for CloudFileBlock {
    fn default() -> Self {
        CloudFileBlock{
            id: Some(0),
            file_block_id: 0,
            cloud_meta_id: 0,
            cloud_file_id: None,
            cloud_file_hash: None,
            status: 0,
            deleted: 0,
            create_time: DateTime::now(),
            update_time: DateTime::now(),
        }
    }
}

impl Default for FileBlockMeta {
    fn default() -> Self {
        FileBlockMeta{
            id: Some(0),
            block_index: 0,
            file_part_id: "".to_string(),
            update_time: 0,
            file_modify_time: 0,
            deleted: 0,
            file_meta_id: 0,
            part_hash: None,
            status: 0,
        }
    }
}
