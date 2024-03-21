// 该代码文件是一个用于百度授权的中间件，用于在请求中添加访问令牌(access token)参数
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

use persistence::CloudMeta;

use crate::baidu::vo::Token;
use crate::storage::TokenProvider;

pub struct BaiduAuthMiddleware {}

impl BaiduAuthMiddleware {
    pub(crate) fn new() -> BaiduAuthMiddleware {
        return BaiduAuthMiddleware {};
    }
}

#[async_trait::async_trait]
impl Middleware for BaiduAuthMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let option: Option<&CloudMeta> = extensions.get();
        if let Some(meta) = option{
            let result = meta.get_token();
            if let Err(e) = result {
                let err = anyhow::Error::msg(e);
                let middleware = reqwest_middleware::Error::Middleware(err);
                return Err(middleware);
            }
            let token: Token = result.unwrap();
            let url = req.url_mut();
            let mut query = String::from(url.query().unwrap_or(""));
            if url.to_string().contains("?") {
                query.push_str("&");
            } else {
                query.push_str("?");
            }
            query.push_str("access_token=");
            query.push_str(token.access_token.as_str());
            url.set_query(Some(query.as_str()));

        }
        next.run(req, extensions).await
    }
}
