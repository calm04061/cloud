use api::ResponseResult;
use persistence::CloudMeta;

use crate::ali::vo::{AuthToken, CompleteRequest, DriveCapacity, UploadPreResult};
use crate::storage::{CreateResponse, Quota, TokenProvider};

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

impl From<CompleteRequest> for CreateResponse {
    fn from(value: CompleteRequest) -> Self {
        CreateResponse {
            encrypt_mode: "".to_string(),
            file_id: value.file_id,
            file_name: "".to_string(),
            file_type: "".to_string(),
        }
    }
}

impl From<UploadPreResult> for CreateResponse {
    fn from(value: UploadPreResult) -> Self {
        CreateResponse {
            encrypt_mode: value.encrypt_mode,
            file_id: value.file_id,
            file_name: value.file_name,
            file_type: value.file_type,
        }
    }
}