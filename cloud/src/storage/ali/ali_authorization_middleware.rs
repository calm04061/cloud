use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;

use crate::domain::table::tables::CloudMeta;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    ding_ding_robot_url: String,
    encourage_desc: String,
    feed_back_switch: bool,
    following_desc: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthToken {
    token_type: String,
    access_token: String,
    pub(crate) refresh_token: Option<String>,
    expires_in: i32,
}

pub struct AliAuthMiddleware;

#[async_trait::async_trait]
impl Middleware for AliAuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let option: Option<&CloudMeta> = extensions.get();
        if let Some(meta) = option {
            let auth_opt = meta.clone().auth;
            if let Some(token) = auth_opt {
                let token: AuthToken = serde_json::from_str(token.as_str()).unwrap();

                let header_map = req.headers_mut();
                let authorization = format!("{} {}", token.token_type, token.access_token);
                header_map.insert("authorization", authorization.parse().unwrap());
            }
        }
        next.run(req, extensions).await
    }
}
