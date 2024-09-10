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
