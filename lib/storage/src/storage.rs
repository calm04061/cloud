use std::future::Future;

use bytes::Bytes;
use log::{debug, error, info};
use reqwest::{Body, Response, StatusCode};
use reqwest_middleware::{ClientWithMiddleware, Error};

use crate::model::AuthMethod::OAuth2;
use crate::model::{AuthMethod, CreateResponse, Quota};
use api::error::ErrorInfo;
use api::ResponseResult;
use http::Extensions;
use persistence::meta::{CloudMeta, FileBlockMeta};
use persistence::MetaStatus;


pub trait TokenProvider<T> {
    fn get_token(&self) -> ResponseResult<T>;
}

pub trait TokenManager {
    fn get_token(&self) -> String;
    fn refresh_token(&self);
}

#[async_trait::async_trait]
pub trait Storage {
    ///
    ///上传文件内容
    ///
    async fn upload_content(
        &mut self,
        file_block: &FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<CreateResponse>;
    ///
    /// 删除文件
    ///
    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()>;
    ///
    /// 读取文件内容
    ///
    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes>;

    ///
    /// 获得容量
    ///
    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota>;
    ///
    /// 获得支持的认证方法
    ///
    fn get_auth_methods(&self) -> Vec<AuthMethod> {
        vec![OAuth2]
    }
    ///
    /// 刷新token
    ///
    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String>;
    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String>;
    async fn callback(&self, _server: String, code: String, cloud_meta: &mut CloudMeta) -> ResponseResult<String>;
    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        cloud_meta.data_root = Some("/app/share-desk".to_string());
        cloud_meta.status = MetaStatus::Enable.into();
        Ok(())
    }
    fn client_id(&self) -> String {
        "client_id".to_string()
    }
    fn client_secret(&self) -> String {
        "client_secret".to_string()
    }
}

#[async_trait::async_trait]
pub trait Network {
    fn get_client(&self) -> &ClientWithMiddleware;
    fn get_api_prefix(&self) -> String;
    async fn do_get_json(
        &self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let resp_result = self
            .get_client()
            .get(format!("{}/{}", self.get_api_prefix(), path))
            .build()?;
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        self.get_response_text(resp_result).await
    }
    async fn do_get_bytes(
        &self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<Bytes> {
        let resp_result = self
            .get_client()
            .get(format!("{}/{}", self.get_api_prefix(), path))
            .build()?;
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        self.get_request_bytes(resp_result).await
    }
    ///
    ///
    ///
    async fn do_post_form(
        &self,
        path: &str,
        form: &Vec<(&str, &str)>,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let resp_result = self
            .get_client()
            .post(format!("{}/{}", self.get_api_prefix(), path))
            .form(form)
            .build()?;
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        self.get_response_text(resp_result).await
    }
    ///
    ///
    ///
    async fn do_put_bytes(
        &self,
        path: &str,
        content: &Vec<u8>,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let vec = content.clone();
        let body = Body::from(vec);
        let resp_result = self
            .get_client()
            .put(format!("{}/{}", self.get_api_prefix(), path))
            .body(body)
            .build()?;
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        self.get_response_text(resp_result).await
    }
    ///
    ///
    ///
    async fn do_delete(
        &self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let resp_result = self
            .get_client()
            .delete(format!("{}/{}", self.get_api_prefix(), path))
            .build()?;
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        self.get_response_text(resp_result).await
    }


    async fn get_request_bytes(
        &self,
        future: impl Future<Output=Result<Response, Error>> + Send,
    ) -> ResponseResult<Bytes> {
        let result = future.await;
        let response = result.unwrap();
        let code = response.status();
        let x = response.url();
        info!("url:{}", x);
        if code.is_redirection() {
            let headers = response.headers();
            let location = headers.get("Location").unwrap();
            let location = location.to_str().unwrap();
            return Err(ErrorInfo::Http302(location.to_string()));
        }
        let body = response.bytes().await;
        match body {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                let option = e.status();
                if let None = option {
                    return Err(ErrorInfo::OTHER(1, "未知错误".to_string()));
                }
                match option {
                    None => Err(ErrorInfo::OTHER(1, "未知错误".to_string())),
                    Some(code) => Err(ErrorInfo::OTHER(1, code.to_string())),
                }
            }
        }
    }

    async fn get_response_text(
        &self,
        future: impl Future<Output=Result<Response, Error>> + Send,
    ) -> ResponseResult<String> {
        debug!("start get_response_text");
        let resp_result = future.await;
        debug!("future get_response_text");
        let json_string_result = match resp_result {
            Ok(resp) => {
                let code = resp.status();
                if code == StatusCode::OK
                    || code == StatusCode::CREATED
                    || code == StatusCode::BAD_REQUEST
                {
                    let x = resp.text();
                    x.await
                } else if code == StatusCode::NO_CONTENT {
                    // info!("NO_CONTENT");
                    Ok(String::new())
                } else if code == StatusCode::FORBIDDEN {
                    // info!("FORBIDDEN");
                    let body = resp.text().await?;
                    return Err(ErrorInfo::Http401(
                        format!("状态码是{},body:{}", code, body),
                    ));
                } else if code == StatusCode::UNAUTHORIZED {
                    // info!("UNAUTHORIZED");
                    let body = resp.text().await?;
                    return Err(ErrorInfo::Http401(
                        format!("状态码是{},body:{}", code, body),
                    ));
                } else if code == StatusCode::NOT_FOUND {
                    let url = resp.url();
                    // info!("NOT_FOUND:{}",url);
                    return Err(ErrorInfo::Http404(url.to_string()));
                } else {
                    info!("error");
                    let url = resp.url();
                    return Err(ErrorInfo::new(
                        code.as_u16() as i32,
                        format!("\n状态码是{}\n{}", code, url).as_str(),
                    ));
                }
            }
            Err(e) => {
                error!("error:{}",e);
                return match e {
                    Error::Middleware(e) => {
                        let result: anyhow::Result<ErrorInfo> = e.downcast();
                        match result {
                            Ok(e) => Err(e),
                            Err(e) => {
                                let string = format!("中间件错误{}", e);
                                Err(ErrorInfo::new(10, string.as_str()))
                            }
                        }
                    }
                    Error::Reqwest(e) => {
                        let url = e.url();
                        let status = e.status();
                        let string = format!("url={:?}:status:{:?}:{}", url, status, e);
                        Err(ErrorInfo::new(20, string.as_str()))
                    }
                };
            }
        };
        match json_string_result {
            Ok(body) => Ok(body),
            Err(e) => {
                let string = format!("{}", e);

                Err(ErrorInfo::new(30, string.as_str()))
            }
        }
    }
}
