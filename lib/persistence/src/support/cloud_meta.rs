use rbs::Value;

use crate::meta::CloudMeta;

impl Default for CloudMeta {
    fn default() -> Self {
        CloudMeta {
            id: None,
            name: "".to_string(),
            auth: Some("{}".to_string()),
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
        map
    }
}
