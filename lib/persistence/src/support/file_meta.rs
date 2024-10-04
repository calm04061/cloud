// use rbs::Value;

use crate::meta::FileMeta;
use chrono::Local;

impl Default for FileMeta {
    fn default() -> Self {
        FileMeta {
            id: None,
            name: "".to_string(),
            parent_id: 0,
            file_type: 0,
            mode: 0,
            gid: 0,
            uid: 0,
            file_length: 0,
            status: 0,
            deleted: 0,
            create_time: Local::now(),
            update_time: Local::now(),
        }
    }
}
