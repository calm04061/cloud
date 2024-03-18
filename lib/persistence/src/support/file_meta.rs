use rbs::Value;

use crate::FileMeta;

impl Default for FileMeta {
    fn default() -> Self {
        FileMeta {
            id: None,
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

impl FileMeta {
    pub fn sync_default() -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "name":"TEXT not null",
            "parent_id":"INT not null",
            "file_type":"int not null",
            "file_length":"int8 not null",
            "status":"int not null",
            "deleted":"int not null",
            "create_time":"int8 not null",
            "update_time":"int8 not null",
        };
        return map;
    }
}
