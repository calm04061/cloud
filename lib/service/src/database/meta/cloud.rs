use rbatis::RBatis;
use rbatis::rbatis_codegen::ops::AsProxy;
use api::error::ErrorInfo;

use api::ResponseResult;
use api::util::IntoOne;
use persistence::CloudMeta;

use crate::database::meta::CloudMetaManager;

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
        Ok(result.unwrap())
    }
    pub async fn select_by_status(&self, status: i8) -> Vec<CloudMeta> {
        CloudMeta::select_by_column(&self.batis.clone(), "status", status)
            .await
            .unwrap()
    }
    pub async fn quota_random(&self, status: i8, size: i32) -> Vec<CloudMeta> {
        CloudMeta::quota_random(&self.batis.clone(), status, size).await.unwrap()
    }
}

impl CloudMetaManager for SimpleCloudMetaManager {
    async fn add(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut meta = meta.clone();
        meta.deleted = i8::from(false);
        let id = CloudMeta::insert(&self.batis.clone(), &meta)
            .await
            .unwrap()
            .last_insert_id
            .i32();
        self.info(id).await
    }

    async fn info(&self, id: i32) -> ResponseResult<CloudMeta> {
        let vec = CloudMeta::select_by_column(&self.batis.clone(), "id", id)
            .await
            .unwrap();
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

        return self.info(meta.id.unwrap()).await;
    }

    async fn delete(&self, id: i32) -> ResponseResult<CloudMeta> {
        let mut meta = self.info(id).await?;
        meta.deleted = i8::from(true);
        CloudMeta::update_by_column(&self.batis.clone(), &meta, "id")
            .await
            .unwrap();
        return self.info(id).await;
    }
}
