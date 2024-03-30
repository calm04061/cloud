use api::util::IntoOne;
use api::ResponseResult;
use persistence::User;
use rbatis::RBatis;

pub struct UserManager {
    batis: RBatis,
}

impl UserManager {
    pub fn new(batis: RBatis) -> Self {
        UserManager { batis }
    }
    pub async fn select_by_username(&self, username: &str) -> ResponseResult<Option<User>> {
        let vec = User::select_by_column(&self.batis.clone(), "username", username).await?;
        if vec.is_empty() {
            return Ok(None);
        }
        Ok(vec.into_one())
    }
}