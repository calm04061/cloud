use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;

use crate::domain::table::tables::CloudMeta;
use crate::storage::ali::vo::AuthToken;
use crate::storage::storage::TokenProvider;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    ding_ding_robot_url: String,
    encourage_desc: String,
    feed_back_switch: bool,
    following_desc: String,
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
            let result = meta.get_token();
            if let Err(e) = result {
                let err = anyhow::Error::msg(e);
                let middleware = reqwest_middleware::Error::Middleware(err);
                return Err(middleware);
            }
            let token: AuthToken = result.unwrap();

            let header_map = req.headers_mut();
            let authorization = format!("{} {}", token.token_type, token.access_token);
            header_map.insert("authorization", authorization.parse().unwrap());
        }
        next.run(req, extensions).await
    }
}
