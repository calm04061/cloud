use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use log::{debug, info};
use reqwest::{Body, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;
use urlencoding::encode;

use crate::domain::table::tables::{CloudMeta, FileBlockMeta};
use crate::error::ErrorInfo;
use crate::storage::ali::ali_authorization_middleware::{AliAuthMiddleware, Token};
use crate::storage::ali::vo::{AliExtra, DevicePersonalInfo, DriveInfo};
use crate::storage::storage::{CloudStorageFile, CreateResponse, FileInfo, FileItemWrapper, Network, OAuthStorageFile, Quota, ResponseResult, SearchResponse, Storage, StorageFile, User};

const CHUNK_SIZE: usize = 10485760;
pub const API_DOMAIN_PREFIX: &str = "https://api.aliyundrive.com";
const AUTH_DOMAIN_PREFIX: &str = "https://openapi.aliyundrive.com";


struct Inner {
    api_client: ClientWithMiddleware,
    content_client: ClientWithMiddleware,
    user: Option<User>,
}

pub struct AliStorage {
    inner: Inner,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DriveFile {
    drive_id: String,
    file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Query {
    drive_id: String,
    parent_file_id: String,
    limit: i32,
    all: bool,
    url_expire_sec: i32,
    image_thumbnail_process: String,
    image_url_process: String,
    video_thumbnail_process: String,
    fields: String,
    order_by: String,
    order_direction: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Search {
    drive_id: String,
    limit: i32,
    order_by: String,
    query: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PartInfo {
    part_number: u32,
    upload_url: Option<String>,
    internal_upload_url: Option<String>,
    content_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateFile {
    drive_id: String,
    parent_file_id: String,
    part_info_list: Vec<PartInfo>,
    name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    file_type: String,
    check_name_mode: Option<String>,
    size: u64,
    pre_hash: String,
    content_hash: Option<String>,
    content_hash_name: Option<String>,
    proof_code: Option<String>,
    proof_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CompleteRequest {
    // {"drive_id":"2050438","upload_id":"617283DD041046B0A97AA79857DDDBBE","file_id":"621cf518a5ddef2ebc7647519486ec82de248fe0"}
    drive_id: String,
    file_id: String,
    upload_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadPreResult {
    parent_file_id: String,
    upload_id: String,
    rapid_upload: bool,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    file_type: String,
    file_id: String,
    domain_id: String,
    drive_id: String,
    encrypt_mode: String,
    file_name: String,
    part_info_list: Vec<PartInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadUrl {
    content_hash: String,
    content_hash_name: String,
    crc64_hash: String,
    expiration: String,
    internal_url: String,
    method: String,
    size: u64,
    url: String,
}

impl AliStorage {
    async fn get_drive_id(&mut self, cloud_meta: CloudMeta) -> ResponseResult<String> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "adrive/v1.0/user/getDriveInfo"))
            .body("{}")
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        // debug!("{:?}", resp_result);
        let json = self.get_response_text(resp_result).await?;
        debug!("{}", json);
        let result: DriveInfo = serde_json::from_str(json.as_str()).unwrap();
        return Ok(result.default_drive_id);
    }
}

impl AliStorage {
    pub(crate) fn new() -> AliStorage {
        let auth_middleware = AliAuthMiddleware::new();
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(auth_middleware).build();

        let content_client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let content_client = ClientBuilder::new(content_client).build();
        AliStorage {
            inner: Inner {
                api_client,
                content_client,
                user: None,
            },
        }
    }
}

#[async_trait::async_trait]
impl Storage for AliStorage {

    async fn user_info(&mut self, cloud_meta: CloudMeta) -> ResponseResult<User> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "v2/user/get"))
            .body("{}")
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        // debug!("{:?}", resp_result);
        let json = self.get_response_text(resp_result).await?;
        debug!("{}", json);
        let result: User = serde_json::from_str(json.as_str()).unwrap();

        self.inner.user = Some(result.clone());
        return Ok(result);
    }
}

impl Clone for AliStorage {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
impl StorageFile for AliStorage {
    async fn upload_content(
        &mut self,
        file_block: FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();

        let len = content.len();
        debug!("content length {}", len);
        let chunk_count = (len as u64 + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64;

        let mut part_infos = vec![];
        let mut index = 0;
        while index < chunk_count {
            part_infos.insert(
                index as usize,
                PartInfo {
                    part_number: (index + 1) as u32,
                    upload_url: None,
                    internal_upload_url: None,
                    content_type: None,
                },
            );
            index += 1;
        }

        let create_file = CreateFile {
            drive_id: drive_id.clone(),
            parent_file_id: cloud_meta.data_root.clone().unwrap(),
            part_info_list: part_infos,
            name: file_block.file_part_id,
            file_type: "file".to_string(),
            check_name_mode: Some("overwrite".to_string()),
            size: len as u64,
            pre_hash: "".to_string(),
            content_hash: None,
            content_hash_name: None,
            proof_code: None,
            proof_version: None,
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v2/file/createWithFolders"
            ))
            .json(&create_file)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.get_response_text(resp_result).await?;
        debug!("createWithFolders:{}", json);
        let result: UploadPreResult = serde_json::from_str(json.as_str()).unwrap();
        let part_info_list = result.part_info_list;

        let mut index = 0;
        let copy = content.clone();
        for part_info in part_info_list {
            let start = index * CHUNK_SIZE;
            let mut end = (index + 1) * CHUNK_SIZE - 1;
            if end > content.len() {
                end = content.len();
            }
            let b = (&copy[start..end]).to_vec();
            let body = Body::from(b);
            let response = self
                .inner.content_client
                .put(part_info.upload_url.unwrap())
                .body(body)
                .send();
            let json = self.get_response_text(response).await?;
            debug!("upload {}", json);
            index += 1;
        }
        let complete = CompleteRequest {
            drive_id: drive_id.clone(),
            file_id: result.file_id,
            upload_id: result.upload_id,
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v2/file/complete"
            ))
            .json(&complete)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.get_response_text(resp_result).await?;
        debug!("complete:{}", json);
        // print!("{:#?}",result);
        return Ok(CreateResponse {
            domain_id: result.domain_id,
            drive_id: result.drive_id,
            encrypt_mode: "".to_string(),
            file_id: complete.file_id,
            file_name: "".to_string(),
            location: "".to_string(),
            parent_file_id: "".to_string(),
            rapid_upload: false,
            file_type: "".to_string(),
            upload_id: "".to_string(),
        });
    }


    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: file_id.to_string(),
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "v2/recyclebin/trash"))
            .json(&scores)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        let result = self.get_response_text(resp_result).await;
        match result {
            Ok(str) => {
                debug!("delete file result :{}", str);
                return Ok(());
            }
            Err(e) => Err(e),
        }
    }

    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        debug!("get_drive_id");
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: file_id.to_string(),
        };
        debug!("get_download_url");
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "v2/file/get_download_url"
            ))
            .json(&scores)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        let json = self.get_response_text(resp_result).await;
        if let Err(e) = json {
            return Err(e);
        }
        let json = json.unwrap();
        let result = serde_json::from_str(json.as_str());
        if let Err(e) = result {
            return Err(ErrorInfo::new_string(102, e.to_string()));
        }
        let url: DownloadUrl = result.unwrap();
        debug!("get_content:{:?}", url);
        let body = self
            .inner.content_client
            .get(url.url)
            .header("Referer", "https://www.aliyundrive.com/")
            .send();
        self.get_request_bytes(body).await
    }



    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let empty: HashMap<String, String> = HashMap::new();
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v1.0/user/getSpaceInfo"
            ))
            .json(&empty)
            .send();

        let json = self.get_response_text(resp_result).await?;

        // println!("{:?}", json);
        let result: Result<DevicePersonalInfo, _> = serde_json::from_str(json.as_str());
        let result: DevicePersonalInfo = result.unwrap();
        return Ok(result.personal_space_info.into());
    }
}
#[async_trait::async_trait]
impl CloudStorageFile for AliStorage {
    async fn info(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        let extra = cloud_meta.extra.unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: file_id.to_string(),
        };
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "v2/file/get"))
            .json(&scores)
            .send();
        let json = self.get_response_text(resp_result).await?;
        let result = serde_json::from_str(json.as_str());
        return Ok(result.unwrap());
    }

    async fn list(
        &mut self,
        parent_file_id: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<FileItemWrapper> {
        let extra = cloud_meta.extra.unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();
        let query = Query {
            drive_id,
            parent_file_id: parent_file_id.to_string(),
            limit: 100,
            all: false,
            url_expire_sec: 1600,
            image_thumbnail_process: "image/resize,w_400/format,jpeg".to_string(),
            image_url_process: "image/resize,w_1920/format,jpeg".to_string(),
            video_thumbnail_process: "video/snapshot,t_1000,f_jpg,ar_auto,w_300".to_string(),
            fields: "*".to_string(),
            order_by: "updated_at".to_string(),
            order_direction: "DESC".to_string(),
        };

        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "adrive/v3/file/list"))
            .json(&query)
            .send();

        let json = self.get_response_text(resp_result).await?;
        // println!("{:?}", json);
        let result = serde_json::from_str(json.as_str());
        return Ok(result.unwrap());
    }

    async fn search(
        &mut self,
        parent_file_id: &str,
        name: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<SearchResponse> {
        let extra = cloud_meta.extra.unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();

        let drive_id = extra.drive_id.unwrap();

        let query = format!(
            "parent_file_id = \"{}\" and (name = \"{}\")",
            parent_file_id, name
        );
        let search = Search {
            drive_id,
            limit: 100,
            order_by: "name ASC".to_string(),
            query,
        };
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "adrive/v3/file/search"))
            .json(&search)
            .send();
        let json = self.get_response_text(resp_result).await?;
        let result = serde_json::from_str(json.as_str());
        return Ok(result.unwrap());
    }
}
#[async_trait::async_trait]
impl OAuthStorageFile for AliStorage{
    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());

        let token_option = cloud_meta.clone().auth;
        let token = token_option.unwrap();
        let token: Token = serde_json::from_str(token.as_str()).unwrap();
        let mut refresh_token = HashMap::new();
        refresh_token.insert("refresh_token", token.refresh_token);

        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "token/refresh"))
            .json(&refresh_token)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        let json = self.get_response_text(resp_result).await?;
        Ok(json)
    }

    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        Ok(format!("{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&scope=user:base,file:all:read&state={}", AUTH_DOMAIN_PREFIX, "iWjfcOWq0BoUNZABxy4hGtXPdftzPtG8", encoded, id))
    }
    async fn callback(&self, server: String, code: String, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, format!("oauth/access_token?grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}", code, "iWjfcOWq0BoUNZABxy4hGtXPdftzPtG8", "KqEOL6F9tT2vkeeYRgKqZvyPHlGQnujM", encoded));
        info!("{}", token_url);
        let mut body = HashMap::new();
        body.insert("client_id", "");
        body.insert("client_secret", "");
        body.insert("grant_type", "authorization_code");
        body.insert("code", code.as_str());
        let resp_result = self.inner.content_client.post(token_url).json(&body).send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }
    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        let result = self.get_drive_id(cloud_meta.clone()).await.unwrap();
        let extra = AliExtra {
            drive_id: Some(result),
        };
        let extra = serde_json::to_string(&extra).unwrap();
        cloud_meta.extra = Some(extra);
        Ok(())
    }

    fn client_id(&self) -> String {
        todo!()
    }

    fn client_secret(&self) -> String {
        todo!()
    }
}

impl Network for AliStorage {
    fn get_client(&self) -> &ClientWithMiddleware {
        &self.inner.api_client
    }

    fn get_api_prefix(&self) -> String {
        API_DOMAIN_PREFIX.to_string()
    }
}

impl Inner {}
