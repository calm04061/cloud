use crate::storage::storage::ResponseResult;
use rbatis::rbatis_codegen::ops::AsProxy;
use rbatis::utils::into_one::IntoOne;

use crate::database::meta::CloudMetaManager;
use crate::domain::table::tables::CloudMeta;
use crate::pool;

#[derive(PartialEq, Debug, Clone)]
pub enum MetaStatus {
    WaitInit,
    WaitDataRoot,
    Enable,
    InvalidRefresh,
    Disabled,
}
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
        meta.status = MetaStatus::WaitInit.into();
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
        // let connection = self.pool.get().unwrap();
        // let mut statement = connection
        //     .prepare("select * from cloud_meta where id=:id")
        //     .unwrap();
        //
        // let result = statement.query_row(
        //     named_params! {
        //         ":id": id,
        //     },
        //     |r| Ok(CloudMeta::from(r)),
        // );
        // return if let Ok(r) = result { Some(r) } else { None };
    }
    async fn update_meta(&self, meta: &CloudMeta) -> Option<CloudMeta> {
        CloudMeta::update_by_column(pool!(), meta, "id")
            .await
            .unwrap();
        // let connection = self.pool.get().unwrap();
        // let mut statement = connection.prepare("update cloud_meta set name =:name,token =:token,last_work_time =:last_work_time,status =:status  where id = :id").unwrap();
        // statement
        //     .execute(named_params! {
        //         ":name":meta.name,
        //         ":token":meta.token,
        //         ":last_work_time":meta.last_work_time,
        //         ":status":meta.status,
        //         ":id":meta.id,
        //     })
        //     .unwrap();
        return self.info(meta.id.unwrap()).await;
    }

    async fn delete(&self, id: i32) -> Option<CloudMeta> {
        let mut meta = self.info(id).await.unwrap();
        meta.deleted = i8::from(true);
        CloudMeta::update_by_column(pool!(), &meta, "id")
            .await
            .unwrap();
        // let connection = self.pool.get().unwrap();
        // let mut statement = connection
        //     .prepare("update cloud_meta set deleted =:deleted where id = :id")
        //     .unwrap();
        // statement
        //     .execute(named_params! {
        //         ":deleted":true,
        //         ":id":id,
        //     })
        //     .unwrap();
        return self.info(id).await;
    }
}
