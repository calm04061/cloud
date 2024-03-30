use rbatis::rbdc::DateTime;
use rbs::Value;

use crate::{CloudFileBlock, FileStatus};

impl CloudFileBlock {
    pub fn init(file_block_id: i32, cloud_meta_id: i32) -> Self {
        CloudFileBlock {
            id: None,
            file_block_id,
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

impl Default for CloudFileBlock {
    fn default() -> Self {
        CloudFileBlock {
            id: None,
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
impl CloudFileBlock {
    pub fn sync_default() -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "file_block_id":"int not null",
            "cloud_meta_id":"int not null",
            "cloud_file_id":"TEXT",
            "cloud_file_hash":"TEXT",
            "status":"int not null",
            "deleted":"int not null",
            "create_time":"int8 not null",
            "update_time":"int8 not null",
        };
        map
    }
}
