use rbatis::RBatis;
use api::ResponseResult;
use api::util::IntoOne;
use persistence::User;

pub struct UserManager {
    batis: RBatis,
}

impl UserManager {
    pub fn new(batis: RBatis) -> Self {
        UserManager { batis }
    }
    pub async fn select_by_username(&self, username: &str) -> ResponseResult<Option<User>> {
        let vec = User::select_by_column(&self.batis.clone(), "username", username).await.unwrap();
        if vec.is_empty() {
            return Ok(None);
        }
        Ok(vec.into_one())
    }
}