use rbs::Value;
use crate::User;

impl User {
    pub fn sync_default() -> Value {
        let map = rbs::to_value! {
            "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
            "username":"TEXT",
            "password":"TEXT",
        };
        return map;
    }
}