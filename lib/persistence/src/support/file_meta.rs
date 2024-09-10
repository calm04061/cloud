// use rbs::Value;

use chrono::Utc;
use crate::meta::FileMeta;

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
            create_time: Utc::now(),
            update_time: Utc::now(),
        }
    }
}

// impl FileMeta {
//     pub fn sync_default() -> Value {
//         let map = rbs::to_value! {
//             "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
//             "name":"TEXT not null",
//             "parent_id":"INT not null",
//             "file_type":"int not null",
//             "mode":"int not null default 0",
//             "gid":"int not null default 0",
//             "uid":"int not null default 0",
//             "file_length":"int8 not null",
//             "status":"int not null",
//             "deleted":"int not null",
//             "create_time":"int8 not null",
//             "update_time":"int8 not null",
//         };
//         map
//     }
// }
