use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use log::info;
use reqwest::{Body, Client};
use reqwest::header::HeaderMap;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Deserialize;
use urlencoding::encode;

use crate::domain::table::tables::{CloudMeta, FileBlockMeta};
use crate::error::ErrorInfo;
use crate::storage::china_mobile::vo::{AccessToken, ChinaMobileResult, DelCatalogContent, DelContentCatalogRes, DiskInfo, PcUploadFileRequest, UploadContentInfo, UploadResult};
use crate::storage::storage::{CreateResponse, Network, OAuthStorageFile, Quota, ResponseResult, Storage, TokenProvider};
use crate::util::{from_xml_default, ToXml};

const CHANNEL_ID: &str = "10009";
const APP_ID: &str = "750894";
const API_DOMAIN_PREFIX: &str = "https://miniapp.yun.139.com";

struct Inner {
    api_client: ClientWithMiddleware,
    // user: Option<VisualProUserInfo>,
}

impl Inner {}

/// http://open.yun.139.com/open/index.html#/help-document-new/index?type=API
/// https://miniapp.yun.139.com/richlifeApp/devapp/getUserInfo
pub struct ChinaMobileStorage {
    // api_client: ClientWithMiddleware,
    // content_client: ClientWithMiddleware,
    // user: Option<User>,
    inner: Inner,
}

impl ChinaMobileStorage {
    pub(crate) fn new() ->Self{
        let content_client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let content_client = ClientBuilder::new(content_client).build();
        ChinaMobileStorage{
            inner: Inner{
                api_client: content_client,
            }
        }
    }
    async fn post_xml<T>(&self, path: &str, cloud_meta: &CloudMeta) -> ResponseResult<T>
        where T: for<'de> Deserialize<'de> {
        return self.post_xml_with_body(path, cloud_meta, None).await;
    }
    async fn post_xml_with_body<T>(&self, path: &str, cloud_meta: &CloudMeta, xml_body: Option<String>) -> ResponseResult<T>
        where T: for<'de> Deserialize<'de> {
        let url = format!("{}{}", API_DOMAIN_PREFIX, path);

        let token: AccessToken = cloud_meta.get_token().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "text/xml;UTF-8".parse().unwrap());
        headers.insert("appid", APP_ID.parse().unwrap());
        headers.insert("secretkey", self.client_id().parse().unwrap());
        headers.insert("accesstoken", token.access_token.parse().unwrap());
        headers.insert("deviceid", "text/xml;UTF-8".parse().unwrap());
        headers.insert("channelid", CHANNEL_ID.parse().unwrap());
        let mut builder = self.inner.api_client.post(url).headers(headers);
        if let Some(t) = xml_body {
            builder = builder.body(t);
        }
        let resp_result = builder.send();
        let body_text = self.inner.get_response_text(resp_result).await.unwrap();
        let result: T = from_xml_default(body_text).unwrap();
        Ok(result)
    }
    async fn do_read_token(&self, url: &str, cloud_meta: &mut CloudMeta, body: &HashMap<&str, &str>) -> ResponseResult<String> {
        let resp_result = self.inner.api_client.post(url).json(&body).send();
        info!("{}", "send");
        let json_text = self.inner.get_response_text(resp_result).await;
        info!("{}", "get_response_text");
        let json_text = match json_text {
            Ok(e) => { e }
            Err(e) => {
                return Err(e);
            }
        };
        info!("{}", json_text);
        let token = serde_json::from_str(json_text.as_str());
        let token: ChinaMobileResult<AccessToken> = match token {
            Ok(token) => { token }
            Err(e) => {
                return Err(ErrorInfo::OTHER(50, e.to_string()));
            }
        };
        let token = token.data.unwrap();
        cloud_meta.expires_in = Some(token.expires_in - 10);
        let token_str = serde_json::to_string(&token).unwrap();
        Ok(token_str)
    }
}

#[async_trait]
impl Storage for ChinaMobileStorage {
    async fn upload_content(&mut self, file_block: &FileBlockMeta, content: &Vec<u8>, cloud_meta: &CloudMeta) -> ResponseResult<CreateResponse> {
        let mut upload_content_list = vec![];
        let upload_content_info = UploadContentInfo {
            content_name: file_block.part_hash.clone(),
            content_size: content.len() as u64,
            content_desc: "".to_string(),
            content_tag_list: "".to_string(),
            comlex_flag: 0,
            comlex_cid: "".to_string(),
            res_cid: "".to_string(),
            digest: "".to_string(),
        };
        upload_content_list.push(upload_content_info);
        let request = PcUploadFileRequest {
            total_size: content.len() as u64,
            upload_content_list,
        };
        let mut body = String::new();
        request.to_xml_with_header(&mut body);

        let result: ChinaMobileResult<UploadResult> = self.post_xml_with_body("/richlifeApp/devapp/IUploadAndDownload", cloud_meta, Some(body)).await.unwrap();
        let result = result.data.unwrap();
        let new_content = result.newContentIDList.get(0).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "image/jpeg; name=11.jpg".parse().unwrap());
        headers.insert("contentSize", "image/jpeg; name=11.jpg".parse().unwrap());
        headers.insert("UploadtaskID", result.uploadTaskID.parse().unwrap());
        let content = content.to_vec();
        let body = Body::from(content);

        let resp_result = self.inner.api_client.post(result.redirectionUrl).headers(headers).body(body).send();
        let body_text = self.inner.get_response_text(resp_result).await.unwrap();
        let _result:Result<(), serde_json::Error> = from_xml_default(body_text);
        Ok(CreateResponse {
            domain_id: "".to_string(),
            drive_id: "".to_string(),
            encrypt_mode: "".to_string(),
            file_id: new_content.contentID.clone(),
            file_name: new_content.contentName.clone(),
            location: "".to_string(),
            parent_file_id: "".to_string(),
            rapid_upload: false,
            file_type: "".to_string(),
            upload_id: "".to_string(),
        })
    }

    async fn delete(&mut self, file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let mut content_id = vec![];
        content_id.push(file_id.to_string());
        let req = DelCatalogContent { content_id };
        let mut xml = String::new();
        req.to_xml_with_header(&mut xml);
        let result: ChinaMobileResult<DelContentCatalogRes> = self.post_xml_with_body("/richlifeApp/devapp/delCatalogContent", cloud_meta, Some(xml)).await.unwrap();
        let _content_info = result.data.unwrap();
        Ok(())
    }

    async fn content(&mut self, _file_id: &str, _cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        todo!()
    }

    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let result: ChinaMobileResult<DiskInfo> = self.post_xml("/richlifeApp/devapp/IUploadAndDownload", cloud_meta).await.unwrap();
        let disk_info = result.data.unwrap();
        Ok(disk_info.into())
    }

    // async fn info(&mut self, file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<FileInfo> {
    //     let req = GetContentInfo {};
    //     let mut xml = String::new();
    //     req.to_xml_with_header(&mut xml);
    //     let result: ChinaMobileResult<ContentInfo> = self.post_xml_with_body("/richlifeApp/devapp/IUploadAndDownload", cloud_meta, Some(xml)).await.unwrap();
    //     let content_info = result.data.unwrap();
    //     Ok(content_info.into())
    // }
}

#[async_trait::async_trait]
impl OAuthStorageFile for ChinaMobileStorage {
    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = format!("{}/open-mpplatform/oauth2/refreshToken", API_DOMAIN_PREFIX);
        info!("{}", token_url);
        let mut body = HashMap::new();
        body.insert("appId", APP_ID);
        let client_secret = self.client_secret();
        body.insert("appSecret", client_secret.as_str());
        let token: AccessToken = cloud_meta.get_token().unwrap();
        body.insert("refreshToken", token.access_token.as_str());
        return self.do_read_token(token_url.as_str(), cloud_meta, &body).await;
    }

    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let string = format!("{}/middle/index.html#/middlePage?pageType=3&app_id={}&appKey={}&deviceId={}&uuid={}&appTitle={}", API_DOMAIN_PREFIX, APP_ID, self.client_id(), id, "", "cloud");
        let call = format!("https://cloud.calm0406.tk/callback.html?target={}&redirect_uri={}", encoded, encode(string.as_str()));
        Ok(call)
    }

    async fn callback(&self, _server: String, uuid: String, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = format!("{}/open-mpplatform/oauth2/accessToken1", API_DOMAIN_PREFIX);
        info!("{}", token_url);
        let mut body = HashMap::new();
        body.insert("uuid", uuid.as_str());
        return self.do_read_token(token_url.as_str(), cloud_meta, &body).await;
    }
    ///
    /// appKey
    ///
    fn client_id(&self) -> String {
        "058FD4CE7A4E38FAE0633FAE990A1AF0".to_string()
    }
    ///
    /// appSecret
    ///
    fn client_secret(&self) -> String {
        "".to_string()
    }
}

impl Network for Inner {
    fn get_client(&self) -> &ClientWithMiddleware {
        &self.api_client
    }
    fn get_api_prefix(&self) -> String {
        API_DOMAIN_PREFIX.to_string()
    }
}