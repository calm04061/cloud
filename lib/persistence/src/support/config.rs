// use rbs::Value;

use crate::Config;

impl Default for Config {
    fn default() -> Self {
        Config {
            id: None,
            property: "".to_string(),
            value: "".to_string(),
        }
    }
}
// impl Config {
//     pub fn sync_default() -> Value {
//         let map = rbs::to_value! {
//             "id":"INTEGER PRIMARY KEY AUTOINCREMENT",
//             "property":"TEXT",
//             "value":"TEXT",
//         };
//         map
//     }
// }