use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use libloading::{Error, Symbol};
use log::{debug, error, info};
use tokio::sync::{Mutex, OnceCell, RwLock};

use api::error::ErrorInfo;
use api::error::ErrorInfo::Http401;
use api::util::IntoOne;
use api::{Capacity, PluginInstance, ResponseResult, CLOUD_FILE_ROOT};
use persistence::meta::{CloudMeta, FileBlockMeta};
use persistence::{CloudFileBlock, CloudType, MetaStatus, TempCloudFile};
use plugin_manager::PLUGIN_MANAGER;
use service::meta::CloudMetaManager;
use service::CONTEXT;

use crate::ali::ali_storage::AliStorage;
use crate::baidu::baidu_storage::BaiduStorage;
use crate::google::google_drive_storage::GoogleDriveStorage;
use crate::local::local_storage::LocalStorage;
use crate::model::AuthMethod;
use crate::onedrive::onedrive_storage::OneDriveStorage;
#[cfg(not(windows))]
use crate::sftp::sftp_storage::SftpStorage;
use crate::storage::Storage;
use crate::web::auth::Callback;

type CloudStorage = Arc<Mutex<dyn Storage + Send>>;

pub struct StorageFacade {
    root: OnceCell<String>,
    plugin_holder: HashMap<CloudType, CloudStorage>,
    account_holder: RwLock<HashMap<i32, CloudStorage>>,
}

impl StorageFacade {
    pub fn new() -> StorageFacade {
        let mut plugin_holder = HashMap::<CloudType, CloudStorage>::new();
        let plugins = PLUGIN_MANAGER.get_plugins();
        for plugin in plugins.iter() {
            let meta_info = &plugin.meta_info;
            for c in meta_info.capacities.iter() {
                if let Capacity::STORAGE(name) = c {
                    match &plugin.instance {
                        PluginInstance::Native(library) => unsafe {
                            info!("loading storage from {}@{}", meta_info.name, meta_info.version);
                            let storage: Result<Symbol<fn() -> (CloudType, Box<CloudStorage>)>, Error> =
                                library.get(name.as_bytes());
                            if let Ok(function) = storage {
                                let (cloud_type, storage) = function();
                                plugin_holder.insert(cloud_type, *storage);
                            }
                        },
                        PluginInstance::Wasi(_) => {
                            info!("WASI plugin {} does not support STORAGE capacity yet", meta_info.name);
                        }
                    }
                }
            }
        }
        StorageFacade {
            root: OnceCell::new(),
            plugin_holder,
            account_holder: RwLock::new(HashMap::new()),
        }
    }

    async fn root(&self) -> ResponseResult<&str> {
        let root = self
            .root
            .get_or_try_init(|| async {
                match CONTEXT.config_manager.info(CLOUD_FILE_ROOT).await {
                    Some(config) => Ok(config.value.clone()),
                    None => Err(ErrorInfo::new(0, "CLOUD_FILE_ROOT config not found")),
                }
            })
            .await?;
        Ok(root.as_str())
    }

    async fn get_cloud(&self, cloud_meta: &CloudMeta) -> ResponseResult<CloudStorage> {
        let cloud_type = CloudType::from(cloud_meta.cloud_type);
        if let Some(cloud) = self.plugin_holder.get(&cloud_type) {
            return Ok(cloud.clone());
        }

        let cloud_meta_id = match cloud_meta.id {
            Some(id) => id,
            None => return Err(ErrorInfo::new(0, "cloud meta id is missing")),
        };

        if let Some(cloud) = self.account_holder.read().await.get(&cloud_meta_id) {
            return Ok(cloud.clone());
        }

        let cloud = self.create_cloud(cloud_type).await?;
        let mut holder = self.account_holder.write().await;
        Ok(holder
            .entry(cloud_meta_id)
            .or_insert_with(|| cloud)
            .clone())
    }

    async fn create_cloud(&self, cloud_type: CloudType) -> ResponseResult<CloudStorage> {
        let root = self.root().await?;
        let cloud: CloudStorage = match cloud_type {
            CloudType::AliYun => Arc::new(Mutex::new(AliStorage::new(root))),
            CloudType::Baidu => Arc::new(Mutex::new(BaiduStorage::new(&root.to_string()))),
            CloudType::Local => Arc::new(Mutex::new(LocalStorage::new(root))),
            CloudType::OneDrive => Arc::new(Mutex::new(OneDriveStorage::new(root))),
            CloudType::GoogleDrive => Arc::new(Mutex::new(GoogleDriveStorage::new(root))),
            #[cfg(not(windows))]
            CloudType::Sftp => Arc::new(Mutex::new(SftpStorage::new(root))),
        };
        Ok(cloud)
    }

    async fn get_meta_info(&self, id: i32) -> ResponseResult<CloudMeta> {
        CONTEXT.cloud_meta_manager.info(id).await
    }

    async fn update_meta_info(&self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CONTEXT.cloud_meta_manager.update_meta(cloud_meta).await?;
        Ok(cloud_meta.clone())
    }

    ///
    /// 授权
    ///
    pub async fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let cloud_meta = self.get_meta_info(id).await?;
        let cloud = self.get_cloud(&cloud_meta).await?;
        let cloud = cloud.lock().await;
        cloud.authorize(server, id)
    }

    ///
    /// 刷新token
    ///
    pub async fn refresh_token(&self, id: i32) -> ResponseResult<CloudMeta> {
        let meta = self.get_meta_info(id).await?;
        self.refresh_meta_token(&meta).await
    }

    ///
    /// 授权登陆回调
    ///
    pub async fn callback(&self, server: &str, callback: &Callback) -> ResponseResult<()> {
        let id = match callback.state.parse::<i32>() {
            Ok(i) => i,
            Err(e) => return Err(ErrorInfo::new(0, &format!("Invalid callback state: {}", e))),
        };
        let mut cloud_meta = self.get_meta_info(id).await?;
        let cloud = self.get_cloud(&cloud_meta).await?;
        let result = {
            let cloud = cloud.lock().await;
            cloud
                .callback(server.to_string(), callback.code.clone(), &mut cloud_meta)
                .await?
        };
        info!("result:{}", result);
        cloud_meta.auth = Some(result);
        cloud_meta.status = MetaStatus::WaitDataRoot.into();
        self.update_meta_info(&cloud_meta).await?;
        self.after_callback(&mut cloud_meta).await;
        self.update_meta_info(&cloud_meta).await?;
        info!("end");
        Ok(())
    }

    ///
    /// 获得支持的授权方式
    ///
    pub async fn get_auth_methods(&self, id: i32) -> Vec<AuthMethod> {
        let cloud_meta = match self.get_meta_info(id).await {
            Ok(m) => m,
            Err(e) => {
                log::error!("Failed to get cloud meta: {}", e);
                return Vec::new();
            }
        };
        let cloud = match self.get_cloud(&cloud_meta).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to get cloud: {}", e);
                return Vec::new();
            }
        };
        let cloud = cloud.lock().await;
        cloud.get_auth_methods()
    }

    ///
    /// 上传文件
    ///
    pub async fn upload_content(
        &self,
        block_meta: &FileBlockMeta,
        cloud_file_block: &mut CloudFileBlock,
        meta: &CloudMeta,
        content: &Vec<u8>,
    ) -> ResponseResult<CloudFileBlock> {
        let result = self
            .upload_content_to_cloud(meta, cloud_file_block, block_meta, content)
            .await;
        if let Ok(result) = result {
            return Ok(result);
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            self.refresh_meta_token(meta).await?;
            return self
                .upload_content_to_cloud(meta, cloud_file_block, block_meta, content)
                .await;
        }
        Err(e)
    }

    async fn upload_content_to_cloud(
        &self,
        cloud_meta: &CloudMeta,
        cloud_file_block: &mut CloudFileBlock,
        block_meta: &FileBlockMeta,
        content: &Vec<u8>,
    ) -> ResponseResult<CloudFileBlock> {
        let cloud_type = CloudType::from(cloud_meta.cloud_type);
        debug!(
            "upload start {} to {:?}({})",
            block_meta.file_part_id, cloud_type, cloud_meta.name
        );
        let cloud = self.get_cloud(cloud_meta).await?;
        let result = cloud
            .lock()
            .await
            .upload_content(block_meta, cloud_file_block, content, cloud_meta)
            .await;
        debug!(
            "upload finish {} to {:?}({})",
            block_meta.file_part_id, cloud_type, cloud_meta.name
        );
        result
    }

    ///
    /// 删除文件
    ///
    pub async fn delete(&self, cloud_file_block: &CloudFileBlock) -> ResponseResult<()> {
        let cloud_meta = self
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await?;
        let cloud_file_id = match &cloud_file_block.cloud_file_id {
            Some(id) => id.as_str(),
            None => return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id)),
        };
        let result = self.delete_from_cloud(cloud_file_id, &cloud_meta).await;
        if let Ok(result) = result {
            return Ok(result);
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            self.refresh_meta_token(&cloud_meta).await?;
            return self.delete_from_cloud(cloud_file_id, &cloud_meta).await;
        }
        Err(e)
    }

    ///
    /// 删除临时文件
    ///
    pub async fn delete_temp_file(&self, cloud_file_block: &TempCloudFile) -> ResponseResult<()> {
        let cloud_meta = self
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await?;
        let cloud_file_id = match &cloud_file_block.cloud_file_id {
            Some(id) => id.as_str(),
            None => return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id)),
        };
        let result = self.delete_from_cloud(cloud_file_id, &cloud_meta).await;
        if let Ok(result) = result {
            return Ok(result);
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            self.refresh_meta_token(&cloud_meta).await?;
            return self.delete_from_cloud(cloud_file_id, &cloud_meta).await;
        }
        Err(e)
    }

    async fn delete_from_cloud(
        &self,
        cloud_file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<()> {
        let cloud_type = CloudType::from(cloud_meta.cloud_type);
        debug!(
            "delete {} from {:?}({})",
            cloud_file_id, cloud_type, cloud_meta.name
        );
        let cloud = self.get_cloud(cloud_meta).await?;
        let result = cloud.lock().await.delete(cloud_file_id, cloud_meta).await;
        result
    }

    ///
    /// 读取文件内容
    ///
    pub async fn content(&self, file_block_id: i32) -> ResponseResult<Bytes> {
        let cloud_file_block = CONTEXT
            .cloud_file_block_manager
            .select_by_file_block_id(file_block_id)
            .await?
            .into_one();
        let cloud_file_block = match cloud_file_block {
            Some(f) => f,
            None => return Err(ErrorInfo::FileNotFound(format!("文件块{file_block_id}不存在"))),
        };
        let cloud_meta = self.get_meta_info(cloud_file_block.cloud_meta_id).await?;
        let cloud_file_id = match &cloud_file_block.cloud_file_id {
            Some(id) => id.as_str(),
            None => return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id)),
        };
        let result = self.content_from_cloud(cloud_file_id, &cloud_meta).await;
        if let Ok(result) = result {
            return Ok(result);
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            self.refresh_meta_token(&cloud_meta).await?;
            return self.content_from_cloud(cloud_file_id, &cloud_meta).await;
        }
        Err(e)
    }

    async fn content_from_cloud(
        &self,
        cloud_file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<Bytes> {
        let cloud = self.get_cloud(cloud_meta).await?;
        let result = cloud.lock().await.content(cloud_file_id, cloud_meta).await;
        result
    }

    ///
    /// 刷新容量
    ///
    pub async fn refresh_quota(&self) -> ResponseResult<()> {
        let status: i8 = MetaStatus::Enable.into();
        let all = CONTEXT.cloud_meta_manager.select_by_status(status).await?;
        for meta in all {
            let result = self.refresh_one_quota(&meta).await;
            if result.is_ok() {
                continue;
            }
            let e = result.err().unwrap();
            if let Http401(_) = e {
                if let Err(e) = self.refresh_meta_token(&meta).await {
                    error!("{}", e);
                    continue;
                }
                if let Err(e) = self.refresh_one_quota(&meta).await {
                    error!("{}", e);
                }
            } else {
                error!("{}", e);
            }
        }
        Ok(())
    }

    async fn refresh_one_quota(&self, meta: &CloudMeta) -> ResponseResult<()> {
        let cloud = self.get_cloud(meta).await?;
        let result = cloud.lock().await.drive_quota(meta).await?;
        let mut meta = meta.clone();
        meta.used_quota = Some(result.used);
        meta.total_quota = Some(result.total);
        meta.remaining_quota = Some(result.remaining);
        CONTEXT.cloud_meta_manager.update_meta(&meta).await?;
        Ok(())
    }

    async fn refresh_meta_token(&self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut cloud_meta = CONTEXT
            .cloud_meta_manager
            .info(cloud_meta.id.unwrap())
            .await?;
        let cloud = self.get_cloud(&cloud_meta).await?;
        let result = {
            let mut cloud = cloud.lock().await;
            cloud.refresh_token(&mut cloud_meta).await
        };
        if let Ok(auth) = result {
            cloud_meta.auth = Some(auth);
            return self.update_meta_info(&cloud_meta).await;
        }
        let error = result.err().unwrap();
        match error {
            ErrorInfo::Http402(m) => {
                cloud_meta.status = MetaStatus::InvalidRefresh.into();
                self.update_meta_info(&cloud_meta).await?;
                Err(ErrorInfo::Http402(m))
            }
            _ => Err(error),
        }
    }

    async fn after_callback(&self, cloud_meta: &mut CloudMeta) {
        let cloud = match self.get_cloud(cloud_meta).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to get cloud for after_callback: {}", e);
                return;
            }
        };
        {
            let mut cloud = cloud.lock().await;
            cloud.after_callback(cloud_meta).await.ok();
        }
        let quota = {
            let mut cloud = cloud.lock().await;
            cloud.drive_quota(cloud_meta).await
        };
        match quota {
            Ok(quota) => {
                cloud_meta.total_quota = Some(quota.total);
                cloud_meta.used_quota = Some(quota.used);
                cloud_meta.remaining_quota = Some(quota.total - quota.used);
            }
            Err(e) => {
                log::error!("Failed to get drive quota: {}", e);
            }
        }
    }
}
