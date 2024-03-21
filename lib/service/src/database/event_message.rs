use rbatis::RBatis;
use rbatis::rbdc::db::ExecResult;

use api::ResponseResult;
use persistence::EventMessage;

pub struct EventMessageManager {
    batis: RBatis
}

impl EventMessageManager {
    pub fn new(batis: RBatis) -> EventMessageManager {
        EventMessageManager{
            batis,
        }
    }
    pub async fn insert(&self, msg: &EventMessage) -> ResponseResult<ExecResult> {
        Ok(EventMessage::insert(&self.batis.clone(),msg).await?)
    }
}