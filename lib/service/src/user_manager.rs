use api::util::IntoOne;
use api::ResponseResult;
use persistence::User;
use crate::DbPool;

pub struct UserManager {
    db_pool: DbPool,
}

impl UserManager {
    pub fn new(db_pool: DbPool) -> Self {
        UserManager { db_pool }
    }
    pub async fn select_by_username(&self, username: &str) -> ResponseResult<Option<User>> {
        let vec = sqlx::query_as("select * from user where username =?")
            .bind(username)
            .fetch_all(&self.db_pool)
            .await?;
        if vec.is_empty() {
            return Ok(None);
        }
        Ok(vec.into_one())
    }
}