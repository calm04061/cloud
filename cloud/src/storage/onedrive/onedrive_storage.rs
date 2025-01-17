use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use dotenv_codegen::dotenv;
use log::info;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use task_local_extensions::Extensions;
use urlencoding::encode;
use persistence::{CloudMeta, FileBlockMeta};

use crate::error::ErrorInfo;
use crate::storage::onedrive::one_drive_authorization_middleware::OneDriveAuthMiddleware;
use crate::storage::onedrive::vo::{AuthorizationToken, Drive, DriveItem};
use crate::storage::storage::{CreateResponse, Network,  Quota, ResponseResult, Storage, User};

const ONE_DRIVE_APP_SECRET: &str = dotenv!("ONE_DRIVE_APP_SECRET");
const ONE_DRIVE_APP_ID: &str = dotenv!("ONE_DRIVE_APP_ID");
pub const API_DOMAIN_PREFIX: &str = "https://graph.microsoft.com/v1.0";
const AUTH_DOMAIN_PREFIX: &str = "https://login.microsoftonline.com";

struct Inner {
    api_client: ClientWithMiddleware,
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

        OneDriveStorage {
            inner: Inner {
                api_client,
                user: None,
            },
        }
    }
}


#[async_trait]
impl Storage for OneDriveStorage {
    async fn upload_content(&mut self, file_block: &FileBlockMeta, content: &Vec<u8>, cloud_meta: &CloudMeta) -> ResponseResult<CreateResponse> {
        let data_root = cloud_meta.data_root.clone().unwrap();
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json(format!("me/drive/root:{}/{}", data_root, file_block.file_part_id).as_str(), &mut extensions)
            .await;
        let path = match json {
            Ok(json) => {
                let drive: DriveItem = serde_json::from_str(&json).unwrap();
                format!("me/drive/items/{}/content", drive.id)
            }
            Err(e) => {
                if let ErrorInfo::Http404(_url) = e {
                    format!("me/drive/root:{}/{}:/content", data_root, file_block.file_part_id)
                } else {
                    return Err(e);
                }
            }
        };
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let x = self.do_put_bytes(path.as_str(), content, &mut extensions).await?;
        let drive: DriveItem = serde_json::from_str(&x)?;
        Ok(drive.into())
    }


    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        self.do_delete(format!("me/drive/items/{}", cloud_file_id).as_str(), &mut extensions).await?;
        Ok(())
    }

    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let result = self
            .do_get_bytes(format!("me/drive/items/{}/content", cloud_file_id).as_str(), &mut extensions)
            .await;
        if let Ok(bo) = result {
            return Ok(bo);
        }
        let e = result.unwrap_err();
        return if let ErrorInfo::Http302(url) = e {
            let resp_result = self
                .inner.api_client
                .get(url.as_str())
                .build()
                .unwrap();
            let resp_result = self
                .get_client()
                .execute(resp_result);
            self.get_request_bytes(resp_result).await
        } else {
            Err(e)
        };
    }


    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .do_get_json("me/drive", &mut extensions)
            .await?;
        info!("{}", json);
        let result: Drive = serde_json::from_str(json.as_str())?;
        let quota = result.quota;
        let user = result.owner.user;
        self.inner.user = Some(user.into());
        return Ok(quota.into());
    }

    async fn refresh_token(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, "consumers/oauth2/v2.0/token".to_string());
        // debug!("{}", token_url);
        let client_id = self.client_id().clone();
        let client_secret = self.client_secret().clone();
        let mut form = vec![];
        let token = cloud_meta.clone().auth.unwrap();
        let token: AuthorizationToken = serde_json::from_str(token.as_str())?;
        let refresh_token = token.refresh_token.unwrap();
        form.push(("grant_type", "refresh_token"));
        form.push(("refresh_token", refresh_token.as_str()));
        form.push(("client_id", client_id.as_str()));
        form.push(("client_secret", client_secret.as_str()));
        let form = form.as_slice();
        let resp_result = self.inner.api_client.post(token_url)
            .form(form)
            .send();
        let json_text = self.get_response_text(resp_result).await?;
        info!("{}", json_text);
        let token: AuthorizationToken = serde_json::from_str(json_text.as_str())?;
        cloud_meta.expires_in = Some(token.expires_in - 10);
        Ok(String::from(json_text))
    }

    ///
    /// https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={client_id}&scope={scope}
    ///     &response_type=token&redirect_uri={redirect_uri}
    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("{}/api/cloud/callback", server);
        let target = encode(callback.as_str());

        let scope = encode("offline_access files.readwrite.all");
        // https://cloud.calm0406.tk/callback.html
        let callback = format!("{}/consumers/oauth2/v2.0/authorize?response_type=code&client_id={}&scope={}&state={}", AUTH_DOMAIN_PREFIX, self.client_id(), scope, id);
        let callback = encode(callback.as_str());
        let result_url = format!("https://cloud.calm0406.tk/callback.html?target={}&redirect_uri={}", target, callback);
        Ok(result_url)
    }
    async fn callback(&self, _server: String, code: String, cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, "consumers/oauth2/v2.0/token");
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
        cloud_meta.expires_in = Some(token.expires_in - 10);
        Ok(String::from(json_text))
    }
    fn client_id(&self) -> String {
        ONE_DRIVE_APP_ID.to_string()
    }

    fn client_secret(&self) -> String {
        ONE_DRIVE_APP_SECRET.to_string()
    }
}

impl Inner {}

impl Network for OneDriveStorage {
    fn get_client(&self) -> &ClientWithMiddleware {
        &self.inner.api_client
    }
    fn get_api_prefix(&self) -> String {
        API_DOMAIN_PREFIX.to_string()
    }
}