use rbs::Value;

use crate::meta::FileBlockMeta;

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
        map
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