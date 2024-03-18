use rbatis::rbatis_codegen::ops::AsProxy;
use persistence::CloudMeta;

use crate::database::meta::CloudMetaManager;
use crate::pool;
use crate::storage::storage::ResponseResult;
use crate::util::IntoOne;


#[derive(PartialEq, Debug, Clone)]
pub struct SimpleCloudMetaManager {}

impl SimpleCloudMetaManager {
    pub(crate) fn new() -> Self {
        SimpleCloudMetaManager {}
    }
    pub(crate) async fn list(&self) -> ResponseResult<Vec<CloudMeta>> {
        let result = CloudMeta::select_by_column(pool!(), "deleted", i8::from(false)).await;
        Ok(result.unwrap())
    }
}

#[async_trait::async_trait]
impl CloudMetaManager for SimpleCloudMetaManager {
    async fn add(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut meta = meta.clone();
        meta.deleted = i8::from(false);
        let id = CloudMeta::insert(pool!(), &meta)
            .await
            .unwrap()
            .last_insert_id
            .i32();
        Ok(self.info(id).await.unwrap())
    }

    async fn info(&self, id: i32) -> Option<CloudMeta> {
        let vec = CloudMeta::select_by_column(pool!(), "id", id)
            .await
            .unwrap();
        if vec.is_empty() {
            return None;
        }
        vec.into_one()
    }
    async fn update_meta(&self, meta: &CloudMeta) -> Option<CloudMeta> {
        CloudMeta::update_by_column(pool!(), meta, "id")
            .await
            .unwrap();

        return self.info(meta.id.unwrap()).await;
    }

    async fn delete(&self, id: i32) -> Option<CloudMeta> {
        let mut meta = self.info(id).await.unwrap();
        meta.deleted = i8::from(true);
        CloudMeta::update_by_column(pool!(), &meta, "id")
            .await
            .unwrap();
        return self.info(id).await;
    }
}
