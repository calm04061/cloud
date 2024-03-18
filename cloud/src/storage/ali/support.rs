use persistence::CloudMeta;
use crate::storage::ali::vo::{AuthToken, DriveCapacity};
use crate::storage::storage::{Quota, ResponseResult, TokenProvider};

impl From<DriveCapacity> for Quota {
    fn from(baidu: DriveCapacity) -> Self {
        Quota {
            total: baidu.total_size,
            used: baidu.used_size,
            remaining: baidu.total_size - baidu.used_size,
        }
    }
}
impl TokenProvider<AuthToken> for CloudMeta {
    fn get_token(&self) -> ResponseResult<AuthToken> {
        let auth_option = self.auth.clone();
        let token = auth_option.unwrap();
        Ok(serde_json::from_str(token.as_str()).unwrap())
    }
}