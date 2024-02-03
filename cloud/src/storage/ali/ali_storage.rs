use std::collections::HashMap;
use std::time::Duration;

use bytes::Bytes;
use dotenv_codegen::dotenv;
use log::{debug, info};
use reqwest::{Body, Client, IntoUrl};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, Serialize};
use task_local_extensions::Extensions;
use urlencoding::encode;
use crate::database::meta::cloud::MetaStatus;
use crate::database::meta::CloudMetaManager;

use crate::domain::table::tables::{CloudMeta, FileBlockMeta};
use crate::error::ErrorInfo;
use crate::service::CONTEXT;
use crate::storage::ali::ali_authorization_middleware::{AliAuthMiddleware};
use crate::storage::ali::vo::{AliExtra, AuthToken, DevicePersonalInfo, DriveInfo, ErrorMessage, FileInfo};
use crate::storage::storage::{CreateResponse, Network, Quota, ResponseResult, Storage, TokenProvider};

//128mb
const CHUNK_SIZE: usize = 134217728;
const ALI_YUN_APP_SECRET: &str = dotenv!("ALI_YUN_APP_SECRET");
const ALI_YUN_APP_ID: &str = dotenv!("ALI_YUN_APP_ID");

pub const API_DOMAIN_PREFIX: &str = "https://openapi.alipan.com";

struct Inner {
    api_client: ClientWithMiddleware,
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

impl From<CompleteRequest> for CreateResponse {
    fn from(value: CompleteRequest) -> Self {
        CreateResponse {
            encrypt_mode: "".to_string(),
            file_id: value.file_id,
            file_name: "".to_string(),
            file_type: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UploadPreResult {
    parent_file_id: String,
    upload_id: Option<String>,
    rapid_upload: bool,
    exist: Option<bool>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    file_type: String,
    file_id: String,
    domain_id: String,
    drive_id: String,
    encrypt_mode: String,
    file_name: String,
    part_info_list: Option<Vec<PartInfo>>,
}

impl From<UploadPreResult> for CreateResponse {
    fn from(value: UploadPreResult) -> Self {
        CreateResponse {
            encrypt_mode: value.encrypt_mode,
            file_id: value.file_id,
            file_name: value.file_name,
            file_type: value.file_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DownloadUrl {
    expiration: String,
    method: String,
    url: String,
}

impl AliStorage {
    async fn get_drive_id(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let json = self.inner.post_api_json("adrive/v1.0/user/getDriveInfo", &(), Some(cloud_meta)).await?;
        debug!("{}", json);
        let result: DriveInfo = serde_json::from_str(json.as_str()).unwrap();
        return Ok(result.default_drive_id);
    }
}

impl AliStorage {
    pub(crate) fn new() -> AliStorage {
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(AliAuthMiddleware).build();
        AliStorage {
            inner: Inner {
                api_client,
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
    async fn create_file(&mut self, create_file: &CreateFile, cloud_meta: &CloudMeta) -> ResponseResult<UploadPreResult> {
        let json = self.inner.post_api_json("adrive/v1.0/openFile/create", create_file, Some(cloud_meta)).await?;
        info!("openFile/create:{}", json);
        let result: UploadPreResult = serde_json::from_str(json.as_str())?;
        Ok(result)
    }

    async fn delete_file(&mut self, file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str()).unwrap();
        let drive_id = extra.drive_id.unwrap();
        let mut body = HashMap::new();
        body.insert("drive_id", drive_id.as_str());
        body.insert("file_id", file_id);
        self.inner.post_api_json("adrive/v1.0/openFile/delete", &body, Some(cloud_meta)).await
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
        let mut extra: AliExtra = serde_json::from_str(extra.as_str())?;
        let root_file_id;
        if extra.root_file_id.is_none() {
            root_file_id = self.resolve_root_dir_not_exist(&cloud_meta.clone()).await?;
            extra.root_file_id = Some(root_file_id.clone());
            let mut meta = cloud_meta.to_owned();
            meta.extra = Some(serde_json::to_string(&extra).unwrap());
            CONTEXT.cloud_meta_manager.update_meta(&meta).await.unwrap();
        } else {
            root_file_id = extra.root_file_id.unwrap();
        }
        let drive_id = extra.drive_id.unwrap();

        let len = content.len();
        info!("content length {}", len);
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
        let mut result = self.create_file(&create_file, cloud_meta).await?;
        if let Some(true) = result.exist {
            self.delete_file(&result.file_id, cloud_meta).await?;
            result = self.create_file(&create_file, cloud_meta).await?;
        }

        let part_info_list = result.part_info_list.unwrap();

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
                .inner.api_client
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
            upload_id: result.upload_id.unwrap(),
        };
        let json = self.inner.post_api_json("adrive/v1.0/openFile/complete", &complete, Some(cloud_meta)).await?;
        debug!("complete:{}", json);
        return Ok(complete.into());
    }


    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;

        let drive_id = extra.drive_id.unwrap();
        let scores = DriveFile {
            drive_id,
            file_id: cloud_file_id.to_string(),
        };
        let result = self.inner.post_api_json("v2/recyclebin/trash", &scores, Some(cloud_meta)).await;
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
        let json = self.inner.post_api_json("adrive/v1.0/openFile/getDownloadUrl", &scores, Some(cloud_meta)).await?;
        let url: DownloadUrl = serde_json::from_str(json.as_str())?;
        debug!("get_content:{:?}", url);
        let body = self
            .inner.api_client
            .get(url.url)
            .header("Referer", "https://www.aliyundrive.com/")
            .send();
        self.inner.get_request_bytes(body).await
    }


    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let json = self.inner.post_api_json("adrive/v1.0/user/getSpaceInfo", &(), Some(cloud_meta)).await?;

        // println!("{:?}", json);
        let result: DevicePersonalInfo = serde_json::from_str(json.as_str())?;
        return Ok(result.personal_space_info.into());
    }

    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token: AuthToken = cloud_meta.get_token().unwrap();
        let mut refresh_token = HashMap::new();
        refresh_token.insert("refresh_token", token.refresh_token.unwrap());
        let client_id = self.client_id();
        let client_secret = self.client_secret();
        refresh_token.insert("client_id", client_id);
        refresh_token.insert("client_secret", client_secret);
        refresh_token.insert("grant_type", "refresh_token".to_string());
        let json = self.inner.post_api_json("oauth/access_token", &refresh_token, None).await?;
        info!("refresh_token result {:?}",json);
        Ok(json)
    }

    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let string = format!("{}/oauth/authorize?response_type=code&client_id={}&scope=user:base,file:all:read,file:all:write&state={}", API_DOMAIN_PREFIX, self.client_id(), id);
        let call = format!("https://cloud.calm0406.tk/callback.html?target={}&redirect_uri={}", encoded, encode(string.as_str()));
        Ok(call)
    }
    async fn callback(&self, server: String, code: String, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let callback = format!("{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let token_url = format!("oauth/access_token?grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}", code, self.client_id(), self.client_secret(), encoded);
        info!("{}", token_url);
        let mut body = HashMap::new();
        let client_id = self.client_id();
        let client_secret = self.client_secret();
        body.insert("client_id", client_id.as_str());
        body.insert("client_secret", client_secret.as_str());
        body.insert("grant_type", "authorization_code");
        body.insert("code", code.as_str());
        let json_text = self.inner.post_api_json(&token_url, &body, None).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }
    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        let result = self.get_drive_id(cloud_meta).await?;
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
        ALI_YUN_APP_ID.to_string()
    }

    fn client_secret(&self) -> String {
        ALI_YUN_APP_SECRET.to_string()
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

        let mut parameter = HashMap::new();
        parameter.insert("drive_id", drive_id);
        parameter.insert("file_path", path.to_string());
        let json = self.post_api_json("adrive/v1.0/openFile/get_by_path", &parameter, Some(&cloud_meta.clone())).await?;
        let result: ErrorMessage = serde_json::from_str(json.as_str())?;
        if let Some(code) = result.code {
            if code.eq("NotFound.File") {
                return Err(ErrorInfo::FileNotFound(path.to_string()));
            }
        }
        info!("path_file_id:{}", json);
        let result: FileInfo = serde_json::from_str(json.as_str())?;
        Ok(result.file_id.unwrap())
    }
    async fn create_dir(&mut self, cloud_meta: &CloudMeta, path: &str) -> ResponseResult<FileInfo> {
        let pos = path.rfind('/');
        let (parent_id, name) = match pos {
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
                (result.unwrap(), &path[(pos + 1)..])
            }
        };
        let extra = cloud_meta.extra.as_ref().unwrap();
        let extra: AliExtra = serde_json::from_str(extra.as_str())?;
        let drive_id = extra.drive_id.unwrap();

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
        let json = self.post_api_json("adrive/v1.0/openFile/create", &create_file, Some(&cloud_meta.clone())).await?;
        info!("createWithFolders:{}", json);
        let result: FileInfo = serde_json::from_str(json.as_str())?;
        Ok(result)
    }
    async fn post_api_json<T: Serialize + ?Sized>(&self, path: &str, body: &T, cloud_meta: Option<&CloudMeta>) -> ResponseResult<String> {
        return self.post_json(format!("{}/{}", API_DOMAIN_PREFIX, path), body, cloud_meta).await;
    }
    async fn post_json<T: Serialize + ?Sized, U: IntoUrl>(&self, url: U, body: &T, cloud_meta: Option<&CloudMeta>) -> ResponseResult<String> {
        let resp_result = self
            .api_client
            .post(url)
            .json(body)
            .build()
            ?;
        if let Some(cloud_meta) = cloud_meta {
            let mut extensions = Extensions::new();
            extensions.insert(cloud_meta.clone());
            let resp_result = self.api_client
                .execute_with_extensions(resp_result, &mut extensions);
            self.get_response_text(resp_result).await
        } else {
            let resp_result = self.api_client.execute(resp_result);
            self.get_response_text(resp_result).await
        }
    }
}
