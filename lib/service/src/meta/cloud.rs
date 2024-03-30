use log::error;
use api::error::ErrorInfo;
use rbatis::rbatis_codegen::ops::AsProxy;
use rbatis::RBatis;
use crate::meta::CloudMetaManager;
use api::util::IntoOne;
use api::ResponseResult;
use persistence::meta::CloudMeta;
use persistence::MetaStatus;

#[derive(Debug, Clone)]
pub struct SimpleCloudMetaManager {
    batis: RBatis,
}

impl SimpleCloudMetaManager {
    pub fn new(batis: RBatis) -> Self {
        SimpleCloudMetaManager { batis }
    }
    pub async fn list(&self) -> ResponseResult<Vec<CloudMeta>> {
        let result = CloudMeta::select_by_column(&self.batis.clone(), "deleted", i8::from(false)).await;
        let result = match result {
            Ok(vec) => Ok(vec),
            Err(e) => Err(e)
        };
        Ok(result?)
    }
    pub async fn select_by_status(&self, status: i8) -> ResponseResult<Vec<CloudMeta>> {
        Ok(CloudMeta::select_by_column(&self.batis.clone(), "status", status)
            .await?)
    }
    pub async fn quota_random(&self, status: i8, size: i32) -> ResponseResult<Vec<CloudMeta>> {
        Ok(CloudMeta::quota_random(&self.batis.clone(), status, size).await?)
    }
    pub async fn query_token_timeout_cloud_meta(&self, now: u64) -> ResponseResult<Vec<CloudMeta>> {
        Ok(CloudMeta::query_token_timeout(&self.batis.clone(), MetaStatus::Enable.into(), now).await?)
    }
}

impl CloudMetaManager for SimpleCloudMetaManager {
    async fn add(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut meta = meta.clone();
        meta.deleted = i8::from(false);
        let id = CloudMeta::insert(&self.batis.clone(), &meta)
            .await?
            .last_insert_id
            .i32();
        self.info(id).await
    }

    async fn info(&self, id: i32) -> ResponseResult<CloudMeta> {
        let vec = CloudMeta::select_by_column(&self.batis.clone(), "id", id)
            .await;
        let vec = match vec {
            Ok(v) => {
                v
            }
            Err(e) => {
                error!("{}",e);
                return Err(ErrorInfo::from(e));
            }
        };
        if vec.is_empty() {
            return Err(ErrorInfo::NoneCloudMeta(id));
        }
        let option = vec.into_one();
        match option {
            Some(meta) => Ok(meta),
            None => Err(ErrorInfo::NoneCloudMeta(id))
        }
    }
    async fn update_meta(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CloudMeta::update_by_column(&self.batis.clone(), meta, "id")
            .await
            ?;

        self.info(meta.id.unwrap()).await
    }

    async fn delete(&self, id: i32) -> ResponseResult<CloudMeta> {
        let mut meta = self.info(id).await?;
        meta.deleted = i8::from(true);
        CloudMeta::update_by_column(&self.batis.clone(), &meta, "id")
            .await?;
        self.info(id).await
    }
}
