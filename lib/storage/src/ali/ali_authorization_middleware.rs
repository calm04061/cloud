use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

use persistence::CloudMeta;

use crate::ali::vo::AuthToken;
use crate::storage::TokenProvider;

pub struct AliAuthMiddleware;

#[async_trait::async_trait]
impl Middleware for AliAuthMiddleware {
    async fn handle(&self, mut req: Request, extensions: &mut Extensions, next: Next<'_>) -> reqwest_middleware::Result<Response> {
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
