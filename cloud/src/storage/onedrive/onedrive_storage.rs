use std::time::Duration;
use bytes::Bytes;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::domain::table::tables::CloudMeta;
use crate::storage::storage::{CreateResponse, FileInfo, FileItemWrapper, Network, Quota, ResponseResult, SearchResponse, Storage, StorageFile, User};
use async_trait::async_trait;
use log::{debug, info};
use task_local_extensions::Extensions;
use urlencoding::encode;
use crate::storage::onedrive::one_drive_authorization_middleware::OneDriveAuthMiddleware;
use crate::storage::onedrive::vo::Drive;

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
    async fn user_info(&mut self, cloud_meta: CloudMeta) -> ResponseResult<User> {
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
        todo!()
    }

    async fn search(&mut self, parent_file_id: &str, name: &str, cloud_meta: CloudMeta) -> ResponseResult<SearchResponse> {
        todo!()
    }

    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()> {
        todo!()
    }

    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        todo!()
    }

    async fn info(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        todo!()
    }

    async fn list(&mut self, parent_file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileItemWrapper> {
        todo!()
    }

    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        todo!()
    }

    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .get_json("/drive", &mut extensions)
            .await?;
        debug!("{}", json);
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
        let encoded = encode(callback.as_str());
        Ok(format!("{}/common/oauth2/v2.0/authorize?response_type=code&client_id={}&redirect_uri={}&scope=onedrive.readwrite&state={}", AUTH_DOMAIN_PREFIX, self.client_id(), encoded, id))
    }

    async fn callback(&self, server: String, code: String, _id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, format!("common/oauth2/v2.0/token?grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}", code, self.client_id(), self.client_secret(), encoded));
        info!("{}", token_url);
        let resp_result = self.inner.content_client.post(token_url).send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }

    fn client_id(&self) -> String {
        "de6e4dde-5b44-4368-a40d-25222a4e1e51".to_string()
    }

    fn client_secret(&self) -> String {
        "KDC8Q~amMqOpYqKe6.DwoH6HXDYZ~UPV0k0Xva3Y".to_string()
    }
}

