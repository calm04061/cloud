use std::future::Future;

use crate::domain::table::tables::CloudMeta;
use bytes::Bytes;
use log::info;
use reqwest::{Response, StatusCode};
use reqwest_middleware::Error;
use serde::{Deserialize, Serialize};

use crate::error::ErrorInfo;

pub type ResponseResult<T> = Result<T, ErrorInfo>;

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
    pub(crate) total: i64,
    pub(crate) used: i64,
    pub(crate) remaining: i64,
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
    pub(crate) domain_id: String,
    pub(crate) drive_id: String,
    pub(crate) encrypt_mode: String,
    pub(crate) file_id: String,
    pub(crate) file_name: String,
    pub(crate) location: String,
    pub(crate) parent_file_id: String,
    pub(crate) rapid_upload: bool,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) upload_id: String,
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
    pub(crate) domain_id: String,
    pub(crate) drive_id: String,
    pub(crate) encrypt_mode: String,
    pub(crate) ex_fields_info: Option<ExFieldsInfo>,
    pub(crate) file_id: String,
    pub(crate) path: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) last_modifier_id: Option<String>,
    pub(crate) last_modifier_name: Option<String>,
    pub(crate) last_modifier_type: Option<String>,
    pub(crate) name: String,
    pub(crate) revision_id: String,
    pub(crate) starred: bool,
    pub(crate) status: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub(crate) file_type: String,
    pub(crate) updated_at: String,
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

pub trait TokenManager {
    fn get_token(&self) -> String;
    fn refresh_token(&self);
}

#[async_trait::async_trait]
pub trait StorageFile {
    // /**
    //  * 上传
    //  **/
    // async fn upload(
    //     &mut self,
    //     parent_file_id: &str,
    //     name: &str,
    //     file_path: &str,
    //     cloud_meta: CloudMeta,
    // ) -> ResponseResult<CreateResponse>;
    /**
     * 上传body
     **/
    async fn upload_content(
        &mut self,
        name: &str,
        content: &Vec<u8>,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<CreateResponse>;
    /**
     * 搜索
     **/
    async fn search(
        &mut self,
        parent_file_id: &str,
        name: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<SearchResponse>;
    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()>;
    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes>;
    async fn info(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileInfo>;
    async fn list(
        &mut self,
        parent_file_id: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<FileItemWrapper>;
    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String>;
    /**
     * 获得容量
     **/
    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota>;
    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String>;
    async fn callback(&self, server: String, code: String, id: i32) -> ResponseResult<String>;
    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()>;
}

#[async_trait::async_trait]
pub trait Storage {
    async fn user_info(&mut self, cloud_meta: CloudMeta) -> ResponseResult<User>;
}

#[async_trait::async_trait]
pub trait Network {
    async fn get_request_bytes(
        &self,
        future: impl Future<Output = Result<Response, Error>> + Send,
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
        // let result: Result<Response, Error> = self.run(future);
        // let future = response.bytes();
        // let result: Result<Bytes, reqwest::Error> = self.run(future);
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

    // async fn get_response_text(
    //     &self,
    //     future: impl Future<Output = Result<Response, Error>> + Send,
    // ) -> ResponseResult<String> {
    //     let resp_result = future.await;
    //     let json_string_result = match resp_result {
    //         Ok(resp) => {
    //             let code = resp.status();
    //             if code == StatusCode::OK
    //                 || code == StatusCode::CREATED
    //                 || code == StatusCode::BAD_REQUEST
    //             {
    //                 let x = resp.text();
    //                 x.await
    //             } else if code == StatusCode::NO_CONTENT {
    //                 Ok(String::new())
    //             } else if code == StatusCode::FORBIDDEN {
    //                 let body = resp.text().await.unwrap();
    //                 return Err(ErrorInfo::new(
    //                     401,
    //                     format!("状态码是{},body:{}", code, body).as_str(),
    //                 ));
    //             } else {
    //                 let url = resp.url();
    //                 return Err(ErrorInfo::new(
    //                     code.as_u16() as i32,
    //                     format!("\n状态码是{}\n{}", code, url).as_str(),
    //                 ));
    //             }
    //         }
    //         Err(e) => {
    //             return match e {
    //                 Error::Middleware(e) => {
    //                     let result: anyhow::Result<ErrorInfo> = e.downcast();
    //                     match result {
    //                         Ok(e) => Err(e),
    //                         Err(e) => {
    //                             let string = format!("中间件错误{}", e);
    //                             Err(ErrorInfo::new(10, string.as_str()))
    //                         }
    //                     }
    //                 }
    //                 Error::Reqwest(e) => {
    //                     let option = e.url();
    //                     let option1 = e.status();
    //                     let string = format!("url={:?}:status:{:?}:{}", option, option1, e);
    //                     Err(ErrorInfo::new(20, string.as_str()))
    //                 }
    //             };
    //         }
    //     };
    //     match json_string_result {
    //         Ok(body) => Ok(body),
    //         Err(e) => {
    //             let string = format!("{}", e);
    //
    //             Err(ErrorInfo::new(30, string.as_str()))
    //         }
    //     }
    // }

    // async fn get_response_json<T: DeserializeOwned>(
    //     &self,
    //     future: impl Future<Output=Result<Response, Error>> + Send,
    // ) -> ResponseResult<T> {
    //     let resp_result = future.await;
    //     if let Err(e) = resp_result {
    //         return match e {
    //             Error::Middleware(e) => {
    //                 let result: anyhow::Result<ErrorInfo> = e.downcast();
    //                 match result {
    //                     Ok(e) => Err(e),
    //                     Err(e) => {
    //                         let string = format!("中间件错误{}", e);
    //                         Err(ErrorInfo::new(10, string.as_str()))
    //                     }
    //                 }
    //             }
    //             Error::Reqwest(e) => {
    //                 let option = e.url();
    //                 let option1 = e.status();
    //                 let string = format!("url={:?}:status:{:?}:{}", option, option1, e);
    //                 Err(ErrorInfo::new(20, string.as_str()))
    //             }
    //         };
    //     }
    //     let resp = resp_result.unwrap();
    //
    //     let code = resp.status();
    //     if code == StatusCode::OK || code == StatusCode::CREATED {
    //         let json = resp.json();
    //         let result = json.await;
    //         if let Ok(t) = result {
    //             return Ok(t);
    //         }
    //         let error = result.err().unwrap();
    //         return Err(ErrorInfo::new(
    //             402,
    //             format!("获得json错误{}", error).as_str(),
    //         ));
    //     }
    //     if code == StatusCode::FORBIDDEN {
    //         return Err(ErrorInfo::new(
    //             401,
    //             format!("状态码是{}", code).as_str(),
    //         ));
    //     } else {
    //         let url = resp.url();
    //         return Err(ErrorInfo::new(
    //             code.as_u16() as i32,
    //             format!("状态码是{},{}", code, url).as_str(),
    //         ));
    //     }
    // }
    async fn get_response_text(
        &self,
        future: impl Future<Output = Result<Response, Error>> + Send,
    ) -> ResponseResult<String> {
        let resp_result = future.await;
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
                    Ok(String::new())
                } else if code == StatusCode::FORBIDDEN {
                    let body = resp.text().await.unwrap();
                    return Err(ErrorInfo::new(
                        401,
                        format!("状态码是{},body:{}", code, body).as_str(),
                    ));
                } else {
                    let url = resp.url();
                    return Err(ErrorInfo::new(
                        code.as_u16() as i32,
                        format!("\n状态码是{}\n{}", code, url).as_str(),
                    ));
                }
            }
            Err(e) => {
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
                        let option = e.url();
                        let option1 = e.status();
                        let string = format!("url={:?}:status:{:?}:{}", option, option1, e);
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