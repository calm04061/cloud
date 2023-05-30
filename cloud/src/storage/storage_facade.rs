use bytes::Bytes;
use rbatis::utils::into_one::IntoOne;

use crate::database::meta::cloud::MetaStatus;
use crate::database::meta::{CloudMetaManager, CloudType};
use crate::domain::table::tables::{CloudFileBlock, CloudMeta};
use crate::error::ErrorInfo;
use crate::error::ErrorInfo::Http401;
use crate::pool;
use crate::service::CONTEXT;
use crate::storage::ali::ali_storage::AliStorage;
use crate::storage::baidu::baidu_storage::BaiduStorage;
use crate::storage::local::local_storage::LocalStorage;
use crate::storage::onedrive::onedrive_storage::OneDriveStorage;
use crate::storage::storage::{CreateResponse, ResponseResult, StorageFile};
use crate::web::vo::auth::Callback;

#[derive(Clone)]
pub struct StorageFacade {
    inner: Inner,
}

#[derive(Clone)]
struct Inner {
    // holder: HashMap<CloudType, Arc<Mutex<dyn StorageFile + Send>>>,
}

impl StorageFacade {
    pub fn new() -> StorageFacade {
        StorageFacade {
            inner: Inner {
                // holder: HashMap::<CloudType, Arc<Mutex<dyn StorageFile + Send>>>::new(),
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
        meta.token = Some(result);
        self.inner.update_meta_info(&meta).await.unwrap();
        Ok(meta)
    }
    ///
    /// 授权登陆回调
    ///
    pub(crate) async fn callback(&mut self, server: &str, callback: &Callback) {
        let id = callback.state.parse().unwrap();
        let mut cloud_meta = self.inner.get_meta_info(id).await.unwrap();

        let result = self
            .inner
            .callback(server, callback, &mut cloud_meta)
            .await
            .unwrap();
        cloud_meta.token = Some(result);
        cloud_meta.status = MetaStatus::WaitDataRoot.into();
        self.inner.update_meta_info(&cloud_meta).await.unwrap();
        self.inner.after_callback(&mut cloud_meta).await;
        self.inner.update_meta_info(&cloud_meta).await.unwrap();
    }
    ///
    /// 上传文件
    ///
    pub(crate) async fn upload_content(
        &mut self,
        name: &str,
        meta: &CloudMeta,
        content: &Vec<u8>,
    ) -> ResponseResult<CreateResponse> {
        let result = self.inner.upload_content(meta, name, content).await;
        if let Ok(o) = result {
            return Ok(o);
        }
        let e = result.err().unwrap();
        return if let Http401(_url) = e {
            let result = self.inner.refresh_token(&meta).await;
            match result {
                Ok(_) => self.inner.upload_content(&meta, name, content).await,
                Err(e) => Err(e),
            }
        } else {
            Err(e)
        }
    }
    ///
    /// 删除文件
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
    pub(crate) async fn content(&mut self, file_block_id: i64) -> ResponseResult<Bytes> {
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
            let mut cloud = self.inner.get_cloud(meta.cloud_type.into());
            // let mut guard = cloud.lock().unwrap();
            let result = cloud.drive_quota(&meta).await.unwrap();
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
    fn get_cloud(&mut self, cloud_type: CloudType) -> Box<dyn StorageFile + Send> {
        let cloud: Box<(dyn StorageFile + Send)> = match cloud_type {
            CloudType::AliYun => Box::new(AliStorage::new()) ,
            CloudType::Baidu => Box::new(BaiduStorage::new()),
            CloudType::Local => Box::new(LocalStorage::new()),
            CloudType::OneDrive => Box::new(OneDriveStorage::new()),
        };
        return cloud;
    }
    async fn get_meta_info(&self, id: i32) -> Option<CloudMeta> {
        return CONTEXT.cloud_meta_manager.info(id).await;
    }
    async fn update_meta_info(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CONTEXT
            .cloud_meta_manager
            .update_meta(cloud_meta)
            .await
            .unwrap();
        Ok(cloud_meta.clone())
    }
    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let mut cloud_meta = CONTEXT
            .cloud_meta_manager
            .info(cloud_meta.id.unwrap())
            .await
            .unwrap();
        let mut cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.refresh_token(&mut cloud_meta).await.unwrap();
        Ok(result)
    }

    pub(crate) async fn upload_content(
        &mut self,
        cloud_meta: &CloudMeta,
        name: &str,
        content: &Vec<u8>,
    ) -> ResponseResult<CreateResponse> {
        // let cloud_meta = self.get_token(1).await.unwrap();

        let mut cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud
            .upload_content(name, &content, cloud_meta.clone())
            .await;
        return result;
    }

    pub(crate) async fn delete(
        &mut self,
        file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<()> {
        let mut cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let mut cloud = cloud.lock().unwrap();
        let result = cloud.delete(file_id, cloud_meta.clone()).await;

        return result;
    }
    // #[async_recursion::async_recursion]
    pub(crate) async fn content(
        &mut self,
        file_id: &str,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<Bytes> {
        // let cloud_meta = self.get_token(1).await.unwrap();

        let mut cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let mut cloud = cloud.lock().unwrap();
        return cloud.content(file_id, cloud_meta.clone()).await;
    }

    pub(crate) async fn authorize(&mut self, server: &str, id: i32) -> String {
        let cloud_meta = CONTEXT.cloud_meta_manager.info(id).await.unwrap();
        let cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let cloud = cloud.lock().unwrap();
        cloud.authorize(server, id).unwrap()
    }

    pub(crate) async fn callback(
        &mut self,
        server: &str,
        callback: &Callback,
        cloud_meta: &mut CloudMeta,
    ) -> ResponseResult<String> {
        let cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let cloud = cloud.lock().unwrap();
        let result = cloud
            .callback(server.to_string(), callback.code.clone(), cloud_meta)
            .await;

        result
    }
    pub(crate) async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) {
        let mut cloud = self.get_cloud(cloud_meta.cloud_type.into());
        // let mut cloud = cloud.lock().unwrap();
        cloud.after_callback(cloud_meta).await.ok();
        let quota = cloud.drive_quota(cloud_meta).await.unwrap();
        cloud_meta.total_quota = Some(quota.total);
        cloud_meta.used_quota = Some(quota.used);
        cloud_meta.remaining_quota = Some(quota.total - quota.used);
    }
}
