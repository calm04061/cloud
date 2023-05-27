use crate::domain::table::tables::CloudMeta;
use crate::error::ErrorInfo;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;

//{"expires_in":2592000,"refresh_token":"122.6d27d0dac1f3e497a2c5ea18b9bb87be.YGOBz1S9vUHf0FjrFG-XFBS2lSxXVDQ9L5UZZUn.l1Imbw","access_token":"121.fc08dafbfbb8fe068a85bccae729cbc7.YgnZRGeB3OdpwwfnkWhUf3rS9WZ-o7ehMBH_5w-.V8c1Iw","session_secret":"","session_key":"","scope":"basic netdisk"}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Token {
    expires_in: i32,
    pub(crate) refresh_token: String,
    access_token: String,
}
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
        next.run(req, extensions).await
    }
}
