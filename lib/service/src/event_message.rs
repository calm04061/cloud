use api::ResponseResult;
use persistence::EventMessage;
use crate::DbPool;

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
        let result = sqlx::query!("insert into event_message (event_type, event_result, message,create_time) values (?,?,?,?)",
            msg.event_type, msg.event_result ,msg.message,msg.create_time )
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
}