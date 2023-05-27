use crate::domain::table::tables::CloudMeta;
use chrono::{DateTime, Local};
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;

use crate::error::ErrorInfo;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    ding_ding_robot_url: String,
    encourage_desc: String,
    feed_back_switch: bool,
    following_desc: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    default_sbox_drive_id: String,
    role: String,
    device_id: Option<String>,
    user_name: String,
    need_link: bool,
    pub(crate) expire_time: DateTime<Local>,
    pin_setup: bool,
    need_rp_verify: bool,
    avatar: String,
    user_data: Option<UserData>,
    token_type: String,
    access_token: String,
    pub(crate) refresh_token: String,
    default_drive_id: String,
    domain_id: String,
    is_first_login: bool,
    user_id: String,
    nick_name: String,
    // exist_link: String,
    state: String,
    expires_in: i32,
    status: String,
}

pub struct AliAuthMiddleware {}

impl AliAuthMiddleware {
    pub(crate) fn new() -> AliAuthMiddleware {
        return AliAuthMiddleware {};
    }
}

#[async_trait::async_trait]
impl Middleware for AliAuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let option: Option<&CloudMeta> = extensions.get();
        if let None = option {
            let err = anyhow::Error::msg(ErrorInfo::NotFoundConfig("db token没有配置".to_string()));
            let middleware = reqwest_middleware::Error::Middleware(err);
            return Err(middleware);
        }
        let meta = option.unwrap();
        let token_option = meta.token.clone();
        if let None = token_option {
            let err = anyhow::Error::msg(ErrorInfo::NotFoundConfig("token没有配置".to_string()));
            let middleware = reqwest_middleware::Error::Middleware(err);
            return Err(middleware);
        }

        let token = token_option.unwrap();
        let token: Token = serde_json::from_str(token.as_str()).unwrap();

        let header_map = req.headers_mut();
        let authorization = format!("{} {}", token.token_type, token.access_token);
        header_map.insert("authorization", authorization.parse().unwrap());
        header_map.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36".parse().unwrap());
        next.run(req, extensions).await
    }
}
