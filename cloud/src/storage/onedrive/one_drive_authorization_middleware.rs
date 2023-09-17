use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

use crate::domain::table::tables::CloudMeta;
use crate::error::ErrorInfo;
use crate::storage::onedrive::vo::AuthorizationToken;
use crate::storage::storage::TokenProvider;

pub struct OneDriveAuthMiddleware {}

impl OneDriveAuthMiddleware {
    pub(crate) fn new() -> OneDriveAuthMiddleware {
        return OneDriveAuthMiddleware {};
    }
}

#[async_trait::async_trait]
impl Middleware for OneDriveAuthMiddleware {
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
        let auth_option = meta.get_token();
        if let Err(e) = auth_option {
            let err = anyhow::Error::msg(e);
            let middleware = reqwest_middleware::Error::Middleware(err);
            return Err(middleware);
        }

        let token: AuthorizationToken = auth_option.unwrap();

        let header_map = req.headers_mut();
        let authorization = format!("{} {}", token.token_type, token.access_token);
        header_map.insert("authorization", authorization.parse().unwrap());
        header_map.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36".parse().unwrap());
        next.run(req, extensions).await
    }
}
