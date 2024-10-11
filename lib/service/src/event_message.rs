use crate::DbPool;
use api::ResponseResult;
use persistence::EventMessage;

pub struct EventMessageManager {
    db_pool: DbPool,
}

impl EventMessageManager {
    pub fn new(db_pool: DbPool) -> EventMessageManager {
        EventMessageManager {
            db_pool,
        }
    }
    pub async fn insert(&self, msg: &EventMessage) -> ResponseResult<i64> {
        let result = sqlx::query("insert into event_message (event_type, event_result, message,create_time) values (?,?,?,?)")
            .bind(&msg.event_type)
            .bind(&msg.event_result )
            .bind(&msg.message)
            .bind(&msg.create_time )
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
}