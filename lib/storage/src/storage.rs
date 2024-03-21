use std::future::Future;

use bytes::Bytes;
use log::{debug, error, info};
use reqwest::{Body, Response, StatusCode};
use reqwest_middleware::{ClientWithMiddleware, Error};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::{CloudMeta, FileBlockMeta, MetaStatus};

use crate::storage::AuthMethod::OAuth2;

#[derive(PartialEq)]
pub enum AuthMethod {
    OAuth2,
    UsernamePassword,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileItemWrapper {
    next_marker: String,
    punished_file_count: i64,
    items: Vec<FileItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ImageMediaMetadata {
    width: i32,
    height: i32,
    image_tags: Vec<ImageTag>,
    image_quality: ImageQuality,
    cropping_suggestion: Vec<CroppingSuggestion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quota {
    pub total: u64,
    pub used: u64,
    pub remaining: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct CroppingSuggestion {
    aspect_ratio: String,
    score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageQuality {
    overall_score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct ImageTag {
    confidence: f64,
    name: String,
    tag_level: i32,
    centric_score: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ExFieldsInfo {
    image_count: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateResponse {
    pub encrypt_mode: String,
    pub file_id: String,
    pub file_name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub file_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileItem {
    create_at: Option<String>,
    creator_id: Option<String>,
    creator_name: Option<String>,
    creator_type: Option<String>,
    domain_id: String,
    drive_id: String,
    encrypt_mode: String,
    file_id: String,
    hidden: bool,
    last_modifier_id: Option<String>,
    last_modifier_name: Option<String>,
    last_modifier_type: Option<String>,
    name: String,
    revision_id: String,
    starred: bool,
    status: String,
    trashed: Option<bool>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    file_type: String,
    updated_at: String,
    user_meta: Option<String>,
    labels: Option<Vec<String>>,
    upload_id: Option<String>,
    parent_file_id: Option<String>,
    crc64_hash: Option<String>,
    content_hash: Option<String>,
    content_hash_name: Option<String>,
    download_url: Option<String>,
    url: Option<String>,
    thumbnail: Option<String>,
    image_media_metadata: Option<ImageMediaMetadata>,
    category: Option<String>,
    punish_flag: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    items: Vec<FileItem>,
    next_marker: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub(crate) create_at: Option<String>,
    pub(crate) creator_id: Option<String>,
    pub(crate) creator_name: Option<String>,
    pub(crate) creator_type: Option<String>,
    pub(crate) domain_id: Option<String>,
    pub(crate) drive_id: Option<String>,
    pub(crate) encrypt_mode: Option<String>,
    pub(crate) ex_fields_info: Option<ExFieldsInfo>,
    pub file_id: String,
    pub(crate) path: Option<String>,
    pub(crate) hidden: Option<bool>,
    pub(crate) last_modifier_id: Option<String>,
    pub(crate) last_modifier_name: Option<String>,
    pub(crate) last_modifier_type: Option<String>,
    pub(crate) name: String,
    pub(crate) revision_id: Option<String>,
    pub(crate) starred: bool,
    pub(crate) status: Option<String>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) updated_at: Option<String>,
    pub(crate) user_meta: Option<String>,

    pub(crate) labels: Option<Vec<String>>,
    pub(crate) upload_id: Option<String>,
    pub(crate) parent_file_id: Option<String>,
    pub(crate) crc64_hash: Option<String>,
    pub(crate) content_hash: Option<String>,
    pub(crate) content_hash_name: Option<String>,
    pub(crate) download_url: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) thumbnail: Option<String>,
    pub(crate) image_media_metadata: Option<ImageMediaMetadata>,
    pub(crate) category: Option<String>,
    pub(crate) punish_flag: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub(crate) domain_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) avatar: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) nick_name: Option<String>,
    pub(crate) phone: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) user_name: Option<String>,
    pub(crate) default_drive_id: Option<String>,
    // "user_data": {},
    pub(crate) deny_change_password_by_self: Option<bool>,
    pub(crate) need_change_password_next_login: Option<bool>,
    // "permission": null
    pub(crate) creator: Option<String>,

    pub(crate) created_at: Option<i64>,
    pub(crate) updated_at: Option<i64>,
}

pub trait TokenProvider<T> {
    fn get_token(&self) -> ResponseResult<T>;
}

pub trait TokenManager {
    fn get_token(&self) -> String;
    fn refresh_token(&self);
}

#[async_trait::async_trait]
pub trait Storage {
    /**
     * 上传body
     **/
    async fn upload_content(
        &mut self,
        file_block: &FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<CreateResponse>;

    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()>;
    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes>;

    /**
     * 获得容量
     **/
    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota>;
    /**
     * 获得支持的认证方法
     **/
    fn get_auth_methods(&self) -> Vec<AuthMethod> {
        vec![OAuth2]
    }
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
            .build()
            .unwrap();
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
    }
    async fn do_get_bytes(
        &self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<Bytes> {
        let resp_result = self
            .get_client()
            .get(format!("{}/{}", self.get_api_prefix(), path))
            .build()
            .unwrap();
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        return self.get_request_bytes(resp_result).await;
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
            .build()
            .unwrap();
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
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
            .build()
            .unwrap();
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
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
            .build()
            .unwrap();
        let resp_result = self
            .get_client()
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
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
                debug!("aa");
                let code = resp.status();
                if code == StatusCode::OK
                    || code == StatusCode::CREATED
                    || code == StatusCode::BAD_REQUEST
                {
                    debug!("bbb");
                    let x = resp.text();
                    x.await
                } else if code == StatusCode::NO_CONTENT {
                    info!("NO_CONTENT");
                    Ok(String::new())
                } else if code == StatusCode::FORBIDDEN {
                    info!("FORBIDDEN");
                    let body = resp.text().await.unwrap();
                    return Err(ErrorInfo::Http401(
                        format!("状态码是{},body:{}", code, body),
                    ));
                } else if code == StatusCode::UNAUTHORIZED {
                    info!("UNAUTHORIZED");
                    let body = resp.text().await.unwrap();
                    return Err(ErrorInfo::Http401(
                        format!("状态码是{},body:{}", code, body),
                    ));
                } else if code == StatusCode::NOT_FOUND {
                    let url = resp.url();
                    info!("NOT_FOUND:{}",url);
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
