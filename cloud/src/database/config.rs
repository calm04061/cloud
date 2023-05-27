use rbatis::utils::into_one::IntoOne;

use crate::domain::table::tables::Config;
use crate::pool;

pub(crate) struct ConfigManager {}

impl ConfigManager {
    pub(crate) fn new() -> Self {
        ConfigManager {}
    }
    pub(crate) async fn info(&self, property: String) -> Option<Config> {
        let vec = Config::select_by_column(pool!(), "property", property)
            .await
            .unwrap();
        if vec.is_empty() {
            None
        } else {
            vec.into_one()
        }
    }
}
