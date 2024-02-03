use rbatis::rbdc::datetime::DateTime;
use rbs::Value;
// use sqlx::{Error, FromRow, Row};
// use sqlx::sqlite::SqliteRow;

use crate::database::meta::{EventResult, EventType, FileStatus};
use crate::domain::table::tables::{CloudFileBlock, CloudMeta, Config, EventMessage, FileBlockMeta, FileMeta};

impl CloudFileBlock {
    pub(crate) fn init(file_block_id: i32, cloud_meta_id: i32) -> Self {
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

impl Default for CloudMeta {
    fn default() -> Self {
        CloudMeta {
            id: None,
            name: "".to_string(),
            auth: None,
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
        Config {
            id: None,
            property: "".to_string(),
            value: "".to_string(),
        }
    }
}


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

impl Default for FileBlockMeta {
    fn default() -> Self {
        FileBlockMeta {
            id: Some(0),
            block_index: 0,
            file_part_id: "".to_string(),
            update_time: 0,
            file_modify_time: 0,
            deleted: 0,
            file_meta_id: 0,
            part_hash: "".to_string(),
            status: 0,
        }
    }
}

impl Config {
    pub fn sync_default() -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "property":"TEXT",
            "value":"TEXT",
        };
        return map;
    }
}

impl CloudMeta {
    pub fn sync_default()  -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "name":"TEXT not null",
            "auth":"TEXT",
            "last_work_time":"int8",
            "data_root":"TEXT",
            "status":"int  not null",
            "deleted":"int not null",
            "cloud_type":"int not null",
            "total_quota":"int8",
            "used_quota":"int8",
            "remaining_quota":"int8",
            "extra":"TEXT",
            "expires_in":"int",
        };
        return map;
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
        return map;
    }
}

impl FileBlockMeta {
    pub fn sync_default()  -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "block_index":"int not null",
            "file_part_id":"text not null",
            "update_time":"int8 not null",
            "file_modify_time":"int8 not null",
            "deleted":"int not null",
            "file_meta_id":"int8 not null",
            "part_hash":"TEXT",
            "status":"int not null",
        };
        return map;
    }
}

impl EventMessage {
    pub(crate) fn sync_default()  -> Value {
            let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "event_type":"int not null",
            "event_result":"text not null",
            "message":"int8 not null",
            "create_time":"int8 not null",
            };
            return map;
    }
    fn new(event_type: EventType, result: EventResult, message: String) -> EventMessage {
        let event_message = EventMessage {
            id: None,
            event_type: event_type.into(),
            event_result: result.into(),
            message,
            create_time: DateTime::now(),

        };
        return event_message;
    }
    pub(crate) fn success(event_type: EventType, message: String) -> EventMessage {
        return EventMessage::new(event_type, EventResult::Success, message);
    }
    pub(crate) fn fail(event_type: EventType, message: String) -> EventMessage {
        return EventMessage::new(event_type, EventResult::Fail, message);
    }
}

// impl FromRow<'_, SqliteRow > for CloudFileBlock {
//     fn from_row(row: &'_ SqliteRow) -> Result<Self, Error> {
//         Ok(CloudFileBlock {
//             id: row.get("id"),
//             file_block_id: row.get("file_block_id"),
//             cloud_meta_id: row.get("cloud_meta_id"),
//             cloud_file_id: row.get("cloud_file_id"),
//             cloud_file_hash: row.get("cloud_file_hash"),
//             status: row.get("status"),
//             deleted: row.get("deleted"),
//             create_time: DateTime::now(),
//             update_time: DateTime::now(),
//         })
//     }
// }