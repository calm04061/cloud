use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use log::{debug, info};
use reqwest::{Body, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;
use urlencoding::encode;
use crate::database::meta::cloud::MetaStatus;

use crate::domain::table::tables::{CloudMeta, FileBlockMeta};
use crate::error::ErrorInfo;
use crate::storage::ali::ali_authorization_middleware::{AliAuthMiddleware, AuthToken};
use crate::storage::ali::vo::{AliExtra, DevicePersonalInfo, DriveInfo, ErrorMessage, FileInfo};
use crate::storage::storage::{CreateResponse, Network, OAuthStorageFile, Quota, ResponseResult, Storage};

const CHUNK_SIZE: usize = 134217728; //128mb
pub const API_DOMAIN_PREFIX: &str = "https://openapi.alipan.com";
const AUTH_DOMAIN_PREFIX: &str = "https://openapi.alipan.com";


struct Inner {
    api_client: ClientWithMiddleware,
    content_client: ClientWithMiddleware,
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
    expiration: String,
    method: String,
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
        let json = self.inner.get_response_text(resp_result).await?;
        debug!("{}", json);
        let result: DriveInfo = serde_json::from_str(json.as_str()).unwrap();
        return Ok(result.default_drive_id);
    }
}

impl AliStorage {
    pub(crate) fn new() -> AliStorage {
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(AliAuthMiddleware).build();

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
            },
        }
    }

    async fn create_dir(&mut self, cloud_meta: &CloudMeta, path: &str) -> ResponseResult<FileInfo> {
        let result = self.inner.create_dir(&cloud_meta.clone(), path).await;
        let result = match result {
            Ok(r) => {
                Ok(r)
            }
            Err(ErrorInfo::FileNotFound(e)) => {
                self.inner.create_dir(&cloud_meta.clone(), e.as_str()).await?;
                self.inner.create_dir(&cloud_meta.clone(), path).await
            }
            Err(e) => {
                Err(e)
            }
        };
        result
    }
    async fn resolve_root_dir_not_exist(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let data_root = cloud_meta.data_root.clone().unwrap();
        let file_id = self.inner.path_file_id(&cloud_meta.clone(), &data_root).await;
        if let Ok(file_id) = file_id {
            return Ok(file_id);
        }
        let info = self.create_dir(&cloud_meta.clone(), &data_root).await?;
        Ok(info.file_id.unwrap())
    }
}


impl Clone for AliStorage {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
impl Storage for AliStorage {
    async fn upload_content(
        &mut self,
        file_block: &FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;
        let root_file_id;
        if extra.root_file_id.is_none() {
            root_file_id = self.resolve_root_dir_not_exist(&cloud_meta.clone()).await?;
        } else {
             root_file_id = extra.root_file_id.unwrap();
        }
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
            parent_file_id: root_file_id,
            part_info_list: part_infos,
            name: file_block.file_part_id.clone(),
            file_type: "file".to_string(),
            check_name_mode: Some("refuse".to_string()),
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
                API_DOMAIN_PREFIX, "adrive/v1.0/openFile/create"
            ))
            .json(&create_file)
            .build()
            ?;
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.inner.get_response_text(resp_result).await?;
        debug!("createWithFolders:{}", json);
        let result: UploadPreResult = serde_json::from_str(json.as_str())?;
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
            let json = self.inner.get_response_text(response).await?;
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
                API_DOMAIN_PREFIX, "adrive/v1.0/openFile/complete"
            ))
            .json(&complete)
            .build()
            .unwrap();
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.inner.get_response_text(resp_result).await?;
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


    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: cloud_file_id.to_string(),
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "v2/recyclebin/trash"))
            .json(&scores)
            .build()?;
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        let result = self.inner.get_response_text(resp_result).await;
        match result {
            Ok(str) => {
                debug!("delete file result :{}", str);
                return Ok(());
            }
            Err(e) => Err(e),
        }
    }

    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        debug!("get_drive_id");
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: cloud_file_id.to_string(),
        };
        debug!("get_download_url");
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v1.0/openFile/getDownloadUrl"
            ))
            .json(&scores)
            .build()?;
        let resp_result = self
            .inner.api_client
            .execute_with_extensions(resp_result, &mut extensions);
        let json = self.inner.get_response_text(resp_result).await?;
        let url: DownloadUrl = serde_json::from_str(json.as_str())?;
        debug!("get_content:{:?}", url);
        let body = self
            .inner.content_client
            .get(url.url)
            .header("Referer", "https://www.aliyundrive.com/")
            .send();
        self.inner.get_request_bytes(body).await
    }


    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let empty: HashMap<String, String> = HashMap::new();
        let resp_result = self
            .inner.api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v1.0/user/getSpaceInfo"
            ))
            .json(&empty)
            .build()?;
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .inner.api_client.execute_with_extensions(resp_result, &mut extensions);
        let json = self.inner.get_response_text(resp_result).await?;

        // println!("{:?}", json);
        let result: DevicePersonalInfo = serde_json::from_str(json.as_str())?;
        return Ok(result.personal_space_info.into());
    }
}

#[async_trait::async_trait]
impl OAuthStorageFile for AliStorage {
    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());

        let token_option = cloud_meta.clone().auth;
        let token = token_option.unwrap();
        let token: AuthToken = serde_json::from_str(token.as_str())?;
        let mut refresh_token = HashMap::new();
        refresh_token.insert("refresh_token", token.refresh_token.unwrap());
        let client_id = self.client_id();
        let client_secret = self.client_secret();
        refresh_token.insert("client_id", client_id);
        refresh_token.insert("client_secret", client_secret);
        refresh_token.insert("grant_type", "refresh_token".to_string());

        let resp_result = self
            .inner.api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "oauth/access_token"))
            .json(&refresh_token)
            .build()?;
        let resp_result = self
            .inner.api_client
            .execute(resp_result);
        let json = self.inner.get_response_text(resp_result).await?;
        let token: AuthToken = serde_json::from_str(json.as_str()).unwrap();
        debug!("{:?}",token);
        Ok(json)
    }

    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let string = format!("{}/oauth/authorize?response_type=code&client_id={}&scope=user:base,file:all:read,file:all:write&state={}", AUTH_DOMAIN_PREFIX, self.client_id(), id);
        let call = format!("https://cloud.calm0406.tk/callback.html?target={}&redirect_uri={}", encoded, encode(string.as_str()));
        Ok(call)
    }
    async fn callback(&self, server: String, code: String, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, format!("oauth/access_token?grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}", code, self.client_id(), self.client_secret(), encoded));
        info!("{}", token_url);
        let mut body = HashMap::new();
        let client_id = self.client_id();
        let client_secret = self.client_secret();
        body.insert("client_id", client_id.as_str());
        body.insert("client_secret", client_secret.as_str());
        body.insert("grant_type", "authorization_code");
        body.insert("code", code.as_str());
        let resp_result = self.inner.content_client.post(token_url).json(&body).send();
        let json_text = self.inner.get_response_text(resp_result).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }
    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        let result = self.get_drive_id(cloud_meta.clone()).await?;
        let extra = AliExtra {
            drive_id: Some(result),
            root_file_id: None,
        };
        let extra = serde_json::to_string(&extra)?;
        cloud_meta.extra = Some(extra);
        cloud_meta.data_root = Some("/app/cluster-shard".to_string());
        cloud_meta.status = MetaStatus::Enable.into();
        Ok(())
    }

    fn client_id(&self) -> String {
        // "386116d55b634aa6be1379ded9e4fdd5".to_string()
        dotenv::var("ALI_YUN_APP_ID").unwrap()
    }

    fn client_secret(&self) -> String {
        // "2fb2bc1a240e47ff909c6721c9efe9e7".to_string()
        dotenv::var("ALI_YUN_APP_SECRET").unwrap()
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

impl Inner {
    async fn path_file_id(&mut self, cloud_meta: &CloudMeta, path: &str) -> ResponseResult<String> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;
        let drive_id = extra.drive_id.unwrap();

        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let mut parameter = HashMap::new();
        parameter.insert("drive_id", drive_id);
        parameter.insert("file_path", path.to_string());
        let resp_result = self
            .api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v1.0/openFile/get_by_path"
            ))
            .json(&parameter)
            .build()
            ?;
        let resp_result = self
            .api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.get_response_text(resp_result).await?;
        let result: ErrorMessage = serde_json::from_str(json.as_str())?;
        if let Some(code) = result.code {
            if code.eq("NotFound.File") {
                return Err(ErrorInfo::FileNotFound(path.to_string()));
            }
        }
        let result: FileInfo = serde_json::from_str(json.as_str())?;
        Ok(result.file_id.unwrap())
    }
    async fn create_dir(&mut self, cloud_meta: &CloudMeta, path: &str) -> ResponseResult<FileInfo> {
        let pos = path.rfind('/');
        let (parent_id,name) = match pos {
            None => {
                ("root".to_string(), path)
            }
            Some(0) => {
                ("root".to_string(), &path[1..])
            }
            Some(pos) => {
                let parent_path = &path[..pos];
                let result = self.path_file_id(&cloud_meta.clone(), parent_path).await;
                if let Err(e) = result {
                    return Err(e);
                }
                (result.unwrap(), &path[(pos+1)..])

            }
        };
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;
        let drive_id = extra.drive_id.unwrap();


        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());

        let create_file = CreateFile {
            drive_id: drive_id.clone(),
            parent_file_id: parent_id,
            part_info_list: vec![],
            name: name.to_string(),
            file_type: "folder".to_string(),
            check_name_mode: Some("refuse".to_string()),
            size: 0,
            pre_hash: "".to_string(),
            content_hash: None,
            content_hash_name: None,
            proof_code: None,
            proof_version: None,
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let resp_result = self
            .api_client
            .post(format!(
                "{}/{}",
                API_DOMAIN_PREFIX, "adrive/v1.0/openFile/create"
            ))
            .json(&create_file)
            .build()
            ?;
        let resp_result = self
            .api_client
            .execute_with_extensions(resp_result, &mut extensions);
        // let resp_result = self.run(resp_result);
        let json = self.get_response_text(resp_result).await?;
        debug!("createWithFolders:{}", json);
        let result: FileInfo = serde_json::from_str(json.as_str())?;
        Ok(result)
    }
}
