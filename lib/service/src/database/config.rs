use rbatis::RBatis;
use api::util::IntoOne;
use persistence::Config;

pub struct ConfigManager {
    batis: RBatis
}

impl ConfigManager {
    pub(crate) fn new(batis: RBatis) -> Self {
        ConfigManager { batis }
    }
    pub async fn info(&self, property: String) -> Option<Config> {
        let vec = Config::select_by_column(&self.batis.clone(), "property", property)
            .await
            .unwrap();
        if vec.is_empty() {
            None
        } else {
            vec.into_one()
        }
    }
}
