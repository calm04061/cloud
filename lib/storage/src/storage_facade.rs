use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use libloading::{Error, Symbol};
use log::{error, info};
use tokio::sync::Mutex;

use api::error::ErrorInfo;
use api::error::ErrorInfo::Http401;
use api::util::IntoOne;
use api::{Capacity, ResponseResult, CLOUD_FILE_ROOT};
use persistence::meta::{CloudMeta, FileBlockMeta};
use persistence::{CloudFileBlock, CloudType, MetaStatus};
use plugin_manager::PLUGIN_MANAGER;
use service::meta::CloudMetaManager;
use service::CONTEXT;

use crate::ali::ali_storage::AliStorage;
use crate::baidu::baidu_storage::BaiduStorage;
use crate::local::local_storage::LocalStorage;
use crate::model::{AuthMethod, CreateResponse};
use crate::onedrive::onedrive_storage::OneDriveStorage;
#[cfg(not(windows))]
use crate::sftp::sftp_storage::SftpStorage;
use crate::storage::Storage;
use crate::web::auth::Callback;

pub struct StorageFacade {
    inner: Inner,
}

struct Inner {
    holder: HashMap<CloudType, Box<Arc<Mutex<dyn Storage + Send>>>>,
    root_holder: Option<String>,
}

impl StorageFacade {
    pub fn new() -> StorageFacade {
        let mut holder = HashMap::<CloudType, Box<Arc<Mutex<dyn Storage + Send>>>>::new();
        let plugins = PLUGIN_MANAGER.get_plugins();
        for plugin in plugins.iter() {
            let meta_info = &plugin.meta_info;
            let cas = meta_info.capacities.iter().clone();
            for c in cas {
                match c {
                    Capacity::STORAGE(name) => unsafe {
                        info!("loading storage from {}@{}",meta_info.name,meta_info.version);
                        let storage: Result<Symbol<fn() -> (CloudType, Box<Arc<Mutex<dyn Storage + Send>>>)>, Error> = plugin.library.get(name.as_bytes());
                        if let Ok(function) = storage {
                            let fun_def = function();
                            holder.insert(fun_def.0, fun_def.1);
                        }
                    }
                    _ => {}
                }
            }
        }
        StorageFacade {
            inner: Inner {
                holder,
                root_holder: None,
            },
        }
    }
}

impl StorageFacade {
    ///
    /// 授权
    ///
    pub async fn authorize(&mut self, server: &str, id: i32) -> ResponseResult<String> {
        self.inner.authorize(server, id).await
    }
    ///
    /// 刷新token
    ///
    pub async fn refresh_token(&mut self, id: i32) -> ResponseResult<CloudMeta> {
        let meta = self.inner.get_meta_info(id).await?;
        self.inner.refresh_token(&meta).await
    }
    ///
    /// 授权登陆回调
    ///
    pub async fn callback(&mut self, server: &str, callback: &Callback) -> ResponseResult<()> {
        let id = callback.state.parse().unwrap();
        let mut cloud_meta = self.inner.get_meta_info(id).await?;

        let result = self
            .inner
            .callback(server, callback, &mut cloud_meta)
            .await?;
        info!("result:{}",result);
        cloud_meta.auth = Some(result);
        cloud_meta.status = MetaStatus::WaitDataRoot.into();
        self.inner.update_meta_info(&cloud_meta).await?;
        self.inner.after_callback(&mut cloud_meta).await;
        self.inner.update_meta_info(&cloud_meta).await?;
        info!("end");
        Ok(())
    }
    ///
    /// 获得支持的授权方式
    ///
    pub async fn get_auth_methods(&mut self, id: i32) -> Vec<AuthMethod> {
        self.inner.get_auth_methods(id).await
    }
    ///
    /// 上传文件
    ///
    pub async fn upload_content(
        &mut self,
        block_meta: &FileBlockMeta,
        meta: &CloudMeta,
        content: &Vec<u8>,
    ) -> ResponseResult<CreateResponse> {
        let result = self.inner.upload_content(meta, &block_meta, content).await;
        if let Ok(o) = result {
            return Ok(o);
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            let result = self.inner.refresh_token(&meta).await;
            match result {
                Ok(_) => self.inner.upload_content(&meta, &block_meta, content).await,
                Err(e) => Err(e),
            }
        } else {
            Err(e)
        }
    }
    ///
    /// 删除文件
    ///
    pub async fn delete(&mut self, cloud_file_block: &CloudFileBlock) -> ResponseResult<()> {
        let cloud_meta = self
            .inner
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await?;
        let cloud_file_id = cloud_file_block.cloud_file_id.clone();
        if let None = cloud_file_id {
            return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id));
        }
        let cloud_file_id = cloud_file_id.unwrap();
        let cloud_file_id = cloud_file_id.as_str();
        let result = self.inner.delete(cloud_file_id, &cloud_meta).await;
        if let Ok(_e) = result {
            return Ok(());
        }
        let e = result.err().unwrap();
        if let Http401(_url) = e {
            let result = self.refresh_token(cloud_meta.id.unwrap()).await;

            match result {
                Ok(_e) => {
                    self.inner.delete(cloud_file_id, &cloud_meta).await
                }
                Err(e) => Err(e),
            }
        } else {
            Err(e)
        }
    }
    ///
    /// 读取文件内容
    ///
    pub async fn content(&mut self, file_block_id: i32) -> ResponseResult<Bytes> {
        let cloud_file_block =
            CONTEXT.cloud_file_block_manager.select_by_file_block_id(file_block_id)
                .await?
                .into_one();
        let cloud_file_block = match cloud_file_block {
            Some(f) => { f }
            None => {
                return Err(ErrorInfo::FileNotFound(format!("文件块{file_block_id}不存在")));
            }
        };
        let cloud_meta = self
            .inner
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await?;
        if let None = cloud_file_block.cloud_file_id {
            return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id));
        }
        let cloud_file_id = cloud_file_block.cloud_file_id.unwrap();
        let cloud_file_id = cloud_file_id.as_str();
        let result = self.inner.content(cloud_file_id, &cloud_meta).await;
        if let Ok(d) = result {
            return Ok(d);
        }
        let e = result.err().unwrap();

        if let Http401(_url) = e {
            let result = self.refresh_token(cloud_meta.id.unwrap()).await;

            match result {
                Ok(_e) => {
                    self.inner.content(cloud_file_id, &cloud_meta).await
                }
                Err(e) => Err(e),
            }
        } else {
            Err(e)
        }
    }
    ///
    /// 刷新容量
    ///
    pub async fn refresh_quota(&mut self) -> ResponseResult<()> {
        let status: i8 = MetaStatus::Enable.into();
        let all = CONTEXT.cloud_meta_manager.select_by_status(status).await?;
        for meta in all {
            let result = self.inner.refresh_quota(meta.clone()).await;
            if result.is_ok() {
                continue;
            }
            let e = result.err().unwrap();
            if let Http401(_) = e {
                let result = self.refresh_token(meta.id.unwrap()).await;
                if let Err(e) = result {
                    error!("{}",e);
                    continue;
                }
                let result = self.inner.refresh_quota(meta.clone()).await;
                if let Err(e) = result {
                    error!("{}",e);
                    continue;
                }
            } else {
                error!("{}",e);
            }
        }
        Ok(())
    }
}

impl Inner {
    ///
    /// 获得云存储
    ///
    async fn get_cloud(&mut self, cloud_type: CloudType) -> ResponseResult<Box<Arc<Mutex<dyn Storage + Send>>>> {
        let root = match self.root_holder.clone() {
            Some(v) => v,
            None => {
                let config = CONTEXT.config_manager.info(CLOUD_FILE_ROOT).await.unwrap();
                self.root_holder = Some(config.value.clone());
                config.value.clone()
            }
        };
        let cloud_storage = self.holder.entry(cloud_type).or_insert_with_key(|key| {
            let cloud: ResponseResult<Box<Arc<Mutex<dyn Storage + Send>>>> = match key {
                CloudType::AliYun => Ok(Box::new(Arc::new(Mutex::new(AliStorage::new(&root))))),
                CloudType::Baidu => Ok(Box::new(Arc::new(Mutex::new(BaiduStorage::new(&root))))),
                CloudType::Local => Ok(Box::new(Arc::new(Mutex::new(LocalStorage::new(&root))))),
                CloudType::OneDrive => Ok(Box::new(Arc::new(Mutex::new(OneDriveStorage::new(&root))))),
                #[cfg(not(windows))]
                CloudType::Sftp => Ok(Box::new(Arc::new(Mutex::new(SftpStorage::new(&root))))),
            };
            cloud.unwrap()
        });
        Ok(cloud_storage.clone())
    }
    ///
    /// 获得元信息
    ///
    async fn get_meta_info(&self, id: i32) -> ResponseResult<CloudMeta> {
        CONTEXT.cloud_meta_manager.info(id).await
    }
    ///
    /// 更新元信息
    ///
    async fn update_meta_info(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CONTEXT
            .cloud_meta_manager
            .update_meta(cloud_meta)
            .await
            ?;
        Ok(cloud_meta.clone())
    }
    ///
    /// 刷新token
    ///
    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut cloud_meta = CONTEXT
            .cloud_meta_manager
            .info(cloud_meta.id.unwrap())
            .await
            ?;
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await?;
        let mut cloud = cloud.lock().await;
        let result = cloud.refresh_token(&mut cloud_meta).await;
        if let Ok(r) = result {
            cloud_meta.auth = Some(r);
            self.update_meta_info(&cloud_meta).await?;
            return Ok(cloud_meta);
        }
        let error = result.err().unwrap();

        match error {
            ErrorInfo::Http402(m) => {
                cloud_meta.status = MetaStatus::InvalidRefresh.into();
                self.update_meta_info(&cloud_meta).await?;
                Err(ErrorInfo::Http402(m))
            }
            _ => {
                Err(error)
            }
        }
    }
    ///
    /// 上传文件
    ///
    pub(crate) async fn upload_content(
        &mut self,
        cloud_meta: &CloudMeta,
        block_meta: &FileBlockMeta,
        content: &Vec<u8>,
    ) -> ResponseResult<CreateResponse> {
        // let cloud_meta = self.get_token(1).await.unwrap();
        let cloud_type = cloud_meta.cloud_type.into();
        info!("upload start {} to {:?}({})", block_meta.file_part_id,cloud_type, cloud_meta.name);
        let cloud = self.get_cloud(cloud_type).await?;
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.lock().await
            .upload_content(&block_meta, &content, &cloud_meta)
            .await;
        info!("upload finish {} to {:?}({})", block_meta.file_part_id,cloud_type, cloud_meta.name);
        result
    }
    ///
    /// 删除文件
    ///
    pub(crate) async fn delete(
        &mut self,
        cloud_file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<()> {
        let cloud_type = cloud_meta.cloud_type.into();

        info!("delete {} from {:?}({})", cloud_file_id,cloud_type, cloud_meta.name);

        let cloud = self.get_cloud(cloud_type).await?;
        let mut cloud = cloud.lock().await;
        let result = cloud.delete(cloud_file_id, &cloud_meta).await;
        result
    }
    ///
    /// 读取文件内容
    ///
    pub(crate) async fn content(
        &mut self,
        cloud_file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<Bytes> {
        // let cloud_meta = self.get_token(1).await.unwrap();

        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await?;
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.lock().await.content(cloud_file_id, &cloud_meta).await;
        result
    }
    ///
    /// 获得支持的授权方式
    ///
    async fn get_auth_methods(&mut self, id: i32) -> Vec<AuthMethod> {
        let cloud_meta = CONTEXT.cloud_meta_manager.info(id).await.unwrap();
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await.unwrap();
        let cloud = cloud.lock().await;
        cloud.get_auth_methods()
    }
    ///
    /// 获得授权地址
    ///
    pub(crate) async fn authorize(&mut self, server: &str, id: i32) -> ResponseResult<String> {
        let cloud_meta = CONTEXT.cloud_meta_manager.info(id).await?;
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await?;
        let cloud = cloud.lock().await;
        cloud.authorize(server, id)
    }
    ///
    /// oauth2回调
    ///
    pub(crate) async fn callback(
        &mut self,
        server: &str,
        callback: &Callback,
        cloud_meta: &mut CloudMeta,
    ) -> ResponseResult<String> {
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await?;
        let cloud = cloud.lock().await;
        let result = cloud
            .callback(server.to_string(), callback.code.clone(), cloud_meta)
            .await;

        result
    }
    ///
    /// 回调后处理
    ///
    pub(crate) async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) {
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).await.unwrap();
        let mut cloud = cloud.lock().await;
        cloud.after_callback(cloud_meta).await.ok();
        let quota = cloud.drive_quota(cloud_meta).await.unwrap();
        cloud_meta.total_quota = Some(quota.total);
        cloud_meta.used_quota = Some(quota.used);
        cloud_meta.remaining_quota = Some(quota.total - quota.used);
    }
    ///
    /// 刷新容量
    ///
    pub async fn refresh_quota(&mut self, mut meta: CloudMeta) -> ResponseResult<()> {
        let cloud = self.get_cloud(meta.cloud_type.into()).await?;
        // let mut guard = cloud.lock().unwrap();
        let result = cloud.lock().await.drive_quota(&meta).await?;
        meta.used_quota = Some(result.used);
        meta.total_quota = Some(result.total);
        meta.remaining_quota = Some(result.remaining);
        CONTEXT.cloud_meta_manager.update_meta(&meta).await?;
        Ok(())
    }
}
