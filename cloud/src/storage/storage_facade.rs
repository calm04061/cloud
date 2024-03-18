use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use log::info;
use tokio::sync::Mutex;
use persistence::{CloudFileBlock, CloudMeta, CloudType, FileBlockMeta, MetaStatus};

use crate::database::meta::{CloudMetaManager};
use crate::error::ErrorInfo;
use crate::error::ErrorInfo::Http401;
use crate::pool;
use crate::service::CONTEXT;
use crate::storage::ali::ali_storage::AliStorage;
use crate::storage::baidu::baidu_storage::BaiduStorage;
use crate::storage::local::local_storage::LocalStorage;
use crate::storage::onedrive::onedrive_storage::OneDriveStorage;
use crate::storage::sftp::sftp_storage::SftpStorage;
use crate::storage::storage::{AuthMethod, CreateResponse, ResponseResult, Storage};
use crate::util::IntoOne;
use crate::web::vo::auth::Callback;

pub struct StorageFacade {
    inner: Inner,
}

struct Inner {
    holder: HashMap<CloudType, Box<Arc<Mutex<dyn Storage + Send>>>>,
}

impl StorageFacade {
    pub fn new() -> StorageFacade {
        StorageFacade {
            inner: Inner {
                holder: HashMap::<CloudType, Box<Arc<Mutex<dyn Storage + Send>>>>::new(),
            },
        }
    }
}

impl StorageFacade {
    ///
    /// 授权
    ///
    pub(crate) async fn authorize(&mut self, server: &str, id: i32) -> String {
        self.inner.authorize(server, id).await
    }
    ///
    /// 刷新token
    ///
    async fn refresh_token(&mut self, id: i32) -> ResponseResult<CloudMeta> {
        let mut meta = self.inner.get_meta_info(id).await.unwrap();
        let result = self.inner.refresh_token(&meta).await.unwrap();
        meta.auth = Some(result);
        self.inner.update_meta_info(&meta).await.unwrap();
        Ok(meta)
    }
    ///
    /// 授权登陆回调
    ///
    pub(crate) async fn callback(&mut self, server: &str, callback: &Callback) -> ResponseResult<()> {
        let id = callback.state.parse().unwrap();
        let mut cloud_meta = self.inner.get_meta_info(id).await.unwrap();

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
    pub(crate) async fn get_auth_methods(&mut self, id: i32) -> Vec<AuthMethod> {
        self.inner.get_auth_methods(id).await
    }
    ///
    /// 上传文件
    ///
    pub(crate) async fn upload_content(
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
        return if let Http401(_url) = e {
            let result = self.inner.refresh_token(&meta).await;
            match result {
                Ok(_) => self.inner.upload_content(&meta, &block_meta, content).await,
                Err(e) => Err(e),
            }
        } else {
            Err(e)
        };
    }
    ///
    /// 删除文件
    ///
    pub(crate) async fn delete(&mut self, cloud_file_block: &CloudFileBlock) -> ResponseResult<()> {
        let cloud_meta = self
            .inner
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await
            .unwrap();
        let cloud_file_id = cloud_file_block.cloud_file_id.clone();
        if let None = cloud_file_id {
            return Err(ErrorInfo::NoneCloudFileId(cloud_file_block.cloud_meta_id));
        }
        let cloud_file_id = cloud_file_id.unwrap();
        let cloud_file_id = cloud_file_id.as_str();
        self.inner.delete(cloud_file_id, &cloud_meta).await
    }
    ///
    /// 读取文件内容
    ///
    pub(crate) async fn content(&mut self, file_block_id: i32) -> ResponseResult<Bytes> {
        let cloud_file_block =
            CloudFileBlock::select_by_column(pool!(), "file_block_id", file_block_id)
                .await
                .unwrap()
                .into_one()
                .unwrap();
        let cloud_meta = self
            .inner
            .get_meta_info(cloud_file_block.cloud_meta_id)
            .await;
        if let None = cloud_meta {
            return Err(ErrorInfo::NoneCloudMeta(cloud_file_block.cloud_meta_id));
        }
        let cloud_meta = cloud_meta.unwrap();
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
                    return self.inner.content(cloud_file_id, &cloud_meta).await;
                }
                Err(e) => Err(e),
            }
        } else {
            return Err(e);
        }
    }
    ///
    /// 刷新容量
    ///
    pub(crate) async fn refresh_quota(&mut self) {
        let status: i8 = MetaStatus::Enable.into();
        let all = CloudMeta::select_by_column(pool!(), "status", status)
            .await
            .unwrap();
        for mut meta in all {
            let cloud = self.inner.get_cloud(meta.cloud_type.into()).unwrap();
            // let mut guard = cloud.lock().unwrap();
            let result = cloud.lock().await.drive_quota(&meta).await.unwrap();
            meta.used_quota = Some(result.used);
            meta.total_quota = Some(result.total);
            meta.remaining_quota = Some(result.remaining);
            CloudMeta::update_by_column(pool!(), &meta, "id")
                .await
                .unwrap();
        }
    }
}

impl Inner {
    ///
    /// 获得云存储
    ///
    fn get_cloud(&mut self, cloud_type: CloudType) -> Result<Box<Arc<Mutex<dyn Storage + Send>>>, &str> {
        let x = self.holder.entry(cloud_type).or_insert_with_key(|key| {
            let cloud: Result<Box<Arc<Mutex<dyn Storage + Send>>>, &str> = match key {
                CloudType::AliYun => Ok(Box::new(Arc::new(Mutex::new(AliStorage::new())))),
                CloudType::Baidu => Ok(Box::new(Arc::new(Mutex::new(BaiduStorage::new())))),
                CloudType::Local => Ok(Box::new(Arc::new(Mutex::new(LocalStorage::new())))),
                CloudType::OneDrive => Ok(Box::new(Arc::new(Mutex::new(OneDriveStorage::new())))),
                CloudType::Sftp => Ok(Box::new(Arc::new(Mutex::new(SftpStorage::new())))),
            };
            cloud.unwrap()
        });
        Ok(x.clone())
    }
    ///
    /// 获得元信息
    ///
    async fn get_meta_info(&self, id: i32) -> Option<CloudMeta> {
        return CONTEXT.cloud_meta_manager.info(id).await;
    }
    ///
    /// 更新元信息
    ///
    async fn update_meta_info(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CONTEXT
            .cloud_meta_manager
            .update_meta(cloud_meta)
            .await
            .unwrap();
        Ok(cloud_meta.clone())
    }
    ///
    /// 刷新token
    ///
    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let mut cloud_meta = CONTEXT
            .cloud_meta_manager
            .info(cloud_meta.id.unwrap())
            .await
            .unwrap();
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
        let mut cloud = cloud.lock().await;
        let result = cloud.refresh_token(&mut cloud_meta).await?;
        cloud_meta.auth = Some(result.clone());
        self.update_meta_info(&cloud_meta).await?;
        Ok(result)
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
        let cloud = self.get_cloud(cloud_type).unwrap();
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.lock().await
            .upload_content(&block_meta, &content, &cloud_meta)
            .await;
        info!("upload finish {} to {:?}({})", block_meta.file_part_id,cloud_type, cloud_meta.name);
        return result;
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

        let cloud = self.get_cloud(cloud_type).unwrap();
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.lock().await.delete(cloud_file_id, &cloud_meta).await;

        return result;
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

        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
        // let mut cloud = cloud.lock().unwrap();
        return cloud.lock().await.content(cloud_file_id, &cloud_meta).await;
    }
    ///
    /// 获得支持的授权方式
    ///
    async fn get_auth_methods(&mut self, id: i32) -> Vec<AuthMethod> {
        let cloud_meta = CONTEXT.cloud_meta_manager.info(id).await.unwrap();
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
        let cloud = cloud.lock().await;
        cloud.get_auth_methods()
    }
    ///
    /// 获得授权地址
    ///
    pub(crate) async fn authorize(&mut self, server: &str, id: i32) -> String {
        let cloud_meta = CONTEXT.cloud_meta_manager.info(id).await.unwrap();
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
        let cloud = cloud.lock().await;
        cloud.authorize(server, id).unwrap()
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
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
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
        let cloud = self.get_cloud(cloud_meta.cloud_type.into()).unwrap();
        let mut cloud = cloud.lock().await;
        cloud.after_callback(cloud_meta).await.ok();
        let quota = cloud.drive_quota(cloud_meta).await.unwrap();
        cloud_meta.total_quota = Some(quota.total);
        cloud_meta.used_quota = Some(quota.used);
        cloud_meta.remaining_quota = Some(quota.total - quota.used);
    }
}
