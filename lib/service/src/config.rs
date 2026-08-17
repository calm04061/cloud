use crate::DbPool;
use api::util::IntoOne;
use persistence::Config;

pub struct ConfigManager {
    db_pool: DbPool,
}

impl ConfigManager {
    pub fn new(db_pool: DbPool) -> Self {
        ConfigManager { db_pool }
    }
    pub async fn info(&self, property: &str) -> Option<Config> {
        let vec = match sqlx::query_as("select * from config where property =?")
            .bind(property)
            .fetch_all(&self.db_pool)
            .await
        {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to query config: {}", e);
                return None;
            }
        };
        if vec.is_empty() {
            None
        } else {
            vec.into_one()
        }
    }
}
