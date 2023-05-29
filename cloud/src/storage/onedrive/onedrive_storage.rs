use std::time::Duration;
use bytes::Bytes;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::domain::table::tables::CloudMeta;
use crate::storage::storage::{CreateResponse, FileInfo, FileItemWrapper, Network, Quota, ResponseResult, SearchResponse, Storage, StorageFile, User};
use async_trait::async_trait;
use log::{info};
use task_local_extensions::Extensions;
use urlencoding::encode;
use crate::error::ErrorInfo;
use crate::storage::onedrive::one_drive_authorization_middleware::OneDriveAuthMiddleware;
use crate::storage::onedrive::vo::{Drive, DriveItem};

pub const API_DOMAIN_PREFIX: &str = "https://graph.microsoft.com/v1.0";
const AUTH_DOMAIN_PREFIX: &str = "https://login.microsoftonline.com";

struct Inner {
    api_client: ClientWithMiddleware,
    content_client: ClientWithMiddleware,
    user: Option<User>,
}

pub(crate) struct OneDriveStorage {
    inner: Inner,
}

impl OneDriveStorage {
    pub fn new() -> Self {
        let auth_middleware = OneDriveAuthMiddleware::new();
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(auth_middleware).build();

        let content_client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let content_client = ClientBuilder::new(content_client).build();
        OneDriveStorage {
            inner: Inner {
                api_client,
                content_client,
                user: None,
            },
        }
    }
}

#[async_trait]
impl Storage for OneDriveStorage {
    async fn user_info(&mut self, _cloud_meta: CloudMeta) -> ResponseResult<User> {
        todo!()
    }
}

impl Network for OneDriveStorage {
    fn get_client(&self) -> &ClientWithMiddleware {
        &self.inner.api_client
    }
    fn get_api_prefix(&self) -> String {
        API_DOMAIN_PREFIX.to_string()
    }
}

#[async_trait]
impl StorageFile for OneDriveStorage {
    async fn upload_content(&mut self, name: &str, content: &Vec<u8>, cloud_meta: CloudMeta) -> ResponseResult<CreateResponse> {
        let data_root = cloud_meta.data_root.clone().unwrap();
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json(format!("me/drive/root:{}/{}", data_root, name).as_str(), &mut extensions)
            .await;
        let path = match json {
            Ok(json) => {
                let drive: DriveItem = serde_json::from_str(&json).unwrap();
                format!("me/drive/items/{}/content", drive.id)
            }
            Err(e) => {
                if let ErrorInfo::Http404(_url) = e {
                    format!("me/drive/root:{}/{}:/content", data_root, name)
                } else {
                    return Err(e);
                }
            }
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let x = self.do_put_bytes(path.as_str(), content, &mut extensions).await.unwrap();
        let drive: DriveItem = serde_json::from_str(&x).unwrap();
        Ok(drive.into())
    }

    async fn search(&mut self, _parent_file_id: &str, _name: &str, _cloud_meta: CloudMeta) -> ResponseResult<SearchResponse> {
        todo!()
    }

    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        self.do_delete(format!("me/drive/items/{}", file_id).as_str(), &mut extensions).await.unwrap();
        Ok(())
    }

    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let result = self
            .do_get_bytes(format!("me/drive/items/{}/content", file_id).as_str(), &mut extensions)
            .await;
        if let Ok(bo)= result{
            return Ok(bo);
        }
        let e = result.unwrap_err();
        return if let ErrorInfo::Http302(url) = e {
            let resp_result = self
                .inner.content_client
                .get(url.as_str())
                .build()
                .unwrap();
            let resp_result = self
                .get_client()
                .execute(resp_result);
            self.get_request_bytes(resp_result).await
        } else {
            Err(e)
        }
    }

    async fn info(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json(format!("me/drive/items/{}", file_id).as_str(), &mut extensions)
            .await
            .unwrap();
        let drive: DriveItem = serde_json::from_str(&json).unwrap();
        Ok(drive.into())
    }

    async fn list(&mut self, _parent_file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<FileItemWrapper> {
        todo!()
    }

    async fn refresh_token(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<String> {
        todo!()
    }

    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json("me/drive", &mut extensions)
            .await?;
        info!("{}", json);
        let result: Drive = serde_json::from_str(json.as_str()).unwrap();
        let quota = result.quota;
        let user = result.owner.user;
        self.inner.user = Some(user.into());
        return Ok(quota.into());
    }
    ///
    /// https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={client_id}&scope={scope}
    ///     &response_type=token&redirect_uri={redirect_uri}
    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let target = encode(callback.as_str());

        let scope = encode("offline_access files.readwrite.all");
        // https://cloud.calm0406.tk/callback.html
        let callback = format!("{}/consumers/oauth2/v2.0/authorize?response_type=code&client_id={}&scope={}&state={}", AUTH_DOMAIN_PREFIX, self.client_id(), scope, id);
        let callback = encode(callback.as_str());
        let result_url = format!("https://cloud.calm0406.tk/callback.html?target={}&redirect_uri={}", target, callback);
        Ok(result_url)
    }

    async fn callback(&self, _server: String, code: String, _id: i32) -> ResponseResult<String> {
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, format!("consumers/oauth2/v2.0/token"));
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
        let resp_result = self.inner.content_client.post(token_url)
            .form(form)
            .send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }

    fn client_id(&self) -> String {
        "2ef3cc2e-2309-4bf4-afb6-918d8177540e".to_string()
    }

    fn client_secret(&self) -> String {
        "iZx8Q~uobOdiWmCdaPIVB4oWfrTAFw5xJ8jXbaXf".to_string()
    }
}

impl Inner {}
