use async_trait::async_trait;
use bytes::Bytes;
use dotenvy;
use http::Extensions;
use log::info;
use reqwest::multipart::Part;
use reqwest::{multipart, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use urlencoding::encode;

use crate::google::google_drive_authorization_middleware::GoogleDriveAuthMiddleware;
use crate::google::vo::{About, AuthorizationToken, Files, GoogleDriveExtra, GoogleDriveFile};
use crate::model::Quota;
use crate::repository::CloudMetaRepository;
use crate::storage::{ExtraStorage, Network, Storage};
use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::meta::{CloudMeta, FileBlockMeta};
use persistence::CloudFileBlock;

fn get_google_drive_app_secret() -> String {
    dotenvy::var("GOOGLE_DRIVE_APP_SECRET").expect("GOOGLE_DRIVE_APP_SECRET not set in environment")
}
fn get_google_drive_app_id() -> String {
    dotenvy::var("GOOGLE_DRIVE_APP_ID").expect("GOOGLE_DRIVE_APP_ID not set in environment")
}
pub const API_DOMAIN_PREFIX: &str = "https://www.googleapis.com";
const AUTH_DOMAIN_PREFIX: &str = "https://accounts.google.com";

struct Inner {
    api_client: ClientWithMiddleware,
}

pub(crate) struct GoogleDriveStorage {
    inner: Inner,
    root: String,
    meta_persister: Arc<dyn CloudMetaRepository>,
}

impl GoogleDriveStorage {
    pub fn new(root: &str, meta_persister: Arc<dyn CloudMetaRepository>) -> Self {
        let auth_middleware = GoogleDriveAuthMiddleware::new();
        let client = Client::builder()
            // .proxy(Proxy::https("http://127.0.0.1:9988").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(auth_middleware).build();

        GoogleDriveStorage {
            inner: Inner {
                api_client,
            },
            root: root.to_string(),
            meta_persister,
        }
    }
    async fn meta(&self, cloud_meta: &CloudMeta, cloud_file_id: &str) -> ResponseResult<GoogleDriveFile> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json(format!("drive/v3/files/{}", cloud_file_id).as_str(), &mut extensions)
            .await?;
        let result = serde_json::from_str(json.as_str())?;
        Ok(result)
    }
    async fn mkdir(&self, cloud_meta: &CloudMeta, name: &str, parent: Option<String>) -> ResponseResult<String> {
        let mut parents = vec![];
        if let Some(value) = parent {
            parents.push(value);
        }
        let file = GoogleDriveFile::new_folder(name, parents);
        let response = self.post_json("https://www.googleapis.com/drive/v3/files", &file, Some(cloud_meta)).await?;
        let file: GoogleDriveFile = serde_json::from_str(response.as_str())?;
        Ok(file.id.unwrap())
    }

    async fn find_dir(&self, cloud_meta: &CloudMeta, name: &str, parent: Option<String>) -> ResponseResult<GoogleDriveFile> {
        let mut query = String::new();
        query.push_str(format!("name='{}'", name).as_str());
        if let Some(value) = parent {
            query.push_str(format!(" and '{}' in parents", value).as_str());
        } else {
            query.push_str(format!(" and '{}' in parents", "root").as_str());
        }
        let query = encode(query.as_str()).to_string();
        let query = format!("q={}", query);
        let response = self.get_json(format!("https://www.googleapis.com/drive/v3/files?&{}", query), Some(cloud_meta)).await?;
        let file: Files = serde_json::from_str(response.as_str())?;
        let vec = file.files;
        if vec.len() == 0 {
            return Err(ErrorInfo::OTHER(1, "".to_string()));
        }
        let x = vec.get(0).unwrap();
        Ok(x.clone())
    }

    async fn resolve_root_dir_not_exist(&self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let root = format!("{}/{}", cloud_meta.data_root.clone().unwrap(), self.root);
        let split = root.split("/");
        let mut parent = None;
        for path in split {
            if path.is_empty() {
                continue;
            }
            let file = self.find_dir(&cloud_meta, path, parent.clone()).await;
            if let Ok(file) = file {
                parent = Some(file.id.unwrap())
            } else {
                parent = Option::from(self.mkdir(cloud_meta, path, parent).await?);
            }
        }
        // parent = Option::from(self.mkdir(cloud_meta, path, parent).await?);
        Ok(parent.unwrap())
    }
}


#[async_trait]
impl Storage for GoogleDriveStorage {
    ///
    /// POST https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart
    /// PUT Location
    async fn upload_content(&mut self, file_block: &FileBlockMeta, cloud_file_block: &mut CloudFileBlock, content: &Vec<u8>, cloud_meta: &CloudMeta) -> ResponseResult<CloudFileBlock> {
        let extra_string = cloud_meta.extra.as_ref().unwrap();
        let mut extra: GoogleDriveExtra = serde_json::from_str(extra_string.as_str())?;
        let root_file_id;
        if extra.root_file_id.is_none() {
            root_file_id = self.resolve_root_dir_not_exist(&cloud_meta.clone()).await?;
            extra.root_file_id = Some(root_file_id.clone());
            let mut meta = cloud_meta.to_owned();
            meta.extra = Some(serde_json::to_string(&extra)?);
            self.meta_persister.update_meta(&meta).await?;
        } else {
            root_file_id = extra.root_file_id.clone().unwrap();
        }

        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let name = &file_block.file_part_id;
        let option = cloud_file_block.cloud_file_id.clone();
        if let Some(cloud_file_id) = option {
            let file = self.meta(cloud_meta, &cloud_file_id).await;
            match file {
                Ok(_file) => {
                    let form = multipart::Form::new();
                    let file = GoogleDriveFile::meta(name, vec![]);
                    let string = serde_json::to_string(&file)?;
                    let meta = Part::text(string).mime_str("application/json; charset=UTF-8")?;
                    let form = form.part("name", meta);
                    let part = Part::bytes(content.clone()).mime_str("application/octet-stream")?;
                    let form = form.part("content", part);
                    let path = format!("upload/drive/v3/files/{}?uploadType=multipart", cloud_file_id);
                    let result = self.do_patch_multipart(path, form, Some(cloud_meta)).await?;
                    println!("{}", result);
                    return Ok(cloud_file_block.clone());
                }
                Err(_) => {}
            }
        }

        let file = GoogleDriveFile::meta(name, vec![root_file_id]);
        let form = multipart::Form::new();
        let string = serde_json::to_string(&file)?;
        let meta = Part::text(string).mime_str("application/json; charset=UTF-8")?;
        let form = form.part("name", meta);
        let part = Part::bytes(content.clone()).mime_str("application/octet-stream")?;
        let form = form.part("content", part);

        let path = "upload/drive/v3/files?uploadType=multipart".to_string();
        let result = self.do_post_multipart(path, form, Some(cloud_meta)).await?;
        let file: GoogleDriveFile = serde_json::from_str(&result)?;
        cloud_file_block.cloud_file_id = Some(file.id.unwrap());
        Ok(cloud_file_block.clone())
    }


    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let result = self.do_delete(format!("drive/v3/files/{cloud_file_id}").as_str(), &mut extensions).await;
        if result.is_ok() {
            return Ok(());
        }
        let e = result.err().unwrap();
        if let ErrorInfo::Http404(_url) = e {
            Ok(())
        } else {
            Err(e)
        }
    }

    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        let result = self.meta(cloud_meta, cloud_file_id).await?;
        let url = result.web_content_link.unwrap();
        let resp_result = self
            .inner.api_client
            .get(url.as_str())
            .build()?;
        let resp_result = self
            .get_client()
            .execute(resp_result);
        self.get_request_bytes(resp_result).await
    }


    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json("drive/v3/about?fields=storageQuota", &mut extensions)
            .await?;
        info!("{}", json);
        let result: About = serde_json::from_str(json.as_str())?;
        let quota = result.storage_quota;
        Ok(quota.into())
    }

    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = "https://oauth2.googleapis.com/token".to_string();
        // debug!("{}", token_url);
        let client_id = self.client_id().clone();
        let client_secret = self.client_secret().clone();
        let mut form = vec![];
        let token = cloud_meta.clone().auth;
        let token: AuthorizationToken = serde_json::from_str(token.unwrap().as_str())?;
        if token.refresh_token.is_none() {
            return Err(ErrorInfo::Http402("refresh_token is none".to_string()));
        }
        let refresh_token = token.refresh_token.unwrap();
        form.push(("grant_type", "refresh_token"));
        form.push(("refresh_token", refresh_token.as_str()));
        form.push(("client_id", client_id.as_str()));
        form.push(("client_id", client_id.as_str()));
        form.push(("client_secret", client_secret.as_str()));
        let form = form.as_slice();
        let resp_result = self.inner.api_client.post(token_url)
            .form(form)
            .send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        let mut token: AuthorizationToken = serde_json::from_str(json_text.as_str())?;
        token.refresh_token = Some(refresh_token);
        let current_time = SystemTime::now();
        let seconds_since_epoch = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        cloud_meta.expires_in = Some((seconds_since_epoch + token.expires_in.unwrap() - 300) as i64);
        Ok(serde_json::to_string(&token)?)
    }

    ///
    /// https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={client_id}&scope={scope}
    ///     &response_type=token&redirect_uri={redirect_uri}
    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("{server}/api/cloud/callback");
        let target = encode(callback.as_str());

        let scope = encode("https://www.googleapis.com/auth/drive");
        // https://cloud.calm0406.tk/callback.html
        let callback = format!("{AUTH_DOMAIN_PREFIX}/o/oauth2/v2/auth?response_type=code&client_id={}&scope={scope}&state={id}&access_type=offline", self.client_id());
        let callback = encode(callback.as_str());
        let result_url = format!("https://cloud.calm0406.tk/callback.html?target={target}&redirect_uri={callback}");
        Ok(result_url)
    }
    async fn callback(&self, _server: String, code: String, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = "https://oauth2.googleapis.com/token".to_string();
        // debug!("{}", token_url);
        let client_id = self.client_id().clone();
        let client_secret = self.client_secret().clone();
        let mut form = vec![];
        form.push(("grant_type", "authorization_code"));
        form.push(("code", code.as_str()));
        form.push(("client_id", client_id.as_str()));
        form.push(("client_secret", client_secret.as_str()));
        form.push(("redirect_uri", "https://cloud.calm0406.tk/callback.html"));
        let form = form.as_slice();
        let resp_result = self.inner.api_client.post(token_url)
            .form(form)
            .send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        let token: AuthorizationToken = serde_json::from_str(json_text.as_str())?;
        let current_time = SystemTime::now();
        let seconds_since_epoch = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        cloud_meta.expires_in = Some((seconds_since_epoch + token.expires_in.unwrap() - 300) as i64);
        Ok(String::from(json_text))
    }

    fn client_id(&self) -> String {
        get_google_drive_app_id()
    }

    fn client_secret(&self) -> String {
        get_google_drive_app_secret()
    }
}

#[async_trait::async_trait]
impl ExtraStorage for GoogleDriveStorage {
    async fn build_extra(&mut self, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let extra = GoogleDriveExtra {
            root_file_id: None,
        };
        let extra = serde_json::to_string(&extra)?;
        Ok(extra)
    }
}

impl Inner {}

impl Network for GoogleDriveStorage {
    fn get_client(&self) -> &ClientWithMiddleware {
        &self.inner.api_client
    }
    fn get_api_prefix(&self) -> String {
        API_DOMAIN_PREFIX.to_string()
    }
}

