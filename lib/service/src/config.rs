
use api::util::IntoOne;
use persistence::Config;
use crate::DbPool;

pub struct ConfigManager {
    db_pool: DbPool,
}

impl ConfigManager {
    pub fn new( db_pool: DbPool) -> Self {
        ConfigManager { db_pool }
    }
    pub async fn info(&self, property: &str) -> Option<Config> {
        let vec = sqlx::query_as("select * from config where property =?")
            .bind(property)
            .fetch_all(&self.db_pool).await.unwrap();
        // let vec = Config::select_by_column(&self.batis.clone(), "property", property)
        //     .await
        //     .unwrap();
        if vec.is_empty() {
            None
        } else {
            vec.into_one()
        }
    }
}
