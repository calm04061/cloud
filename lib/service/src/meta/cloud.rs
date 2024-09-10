use log::error;
use api::error::ErrorInfo;
use crate::meta::CloudMetaManager;
use api::util::IntoOne;
use api::ResponseResult;
use persistence::meta::CloudMeta;
use persistence::MetaStatus;
use crate::DbPool;

#[derive(Debug, Clone)]
pub struct SimpleCloudMetaManager {
    db_pool: DbPool,
}

impl SimpleCloudMetaManager {
    pub fn new(db_pool: DbPool) -> Self {
        SimpleCloudMetaManager { db_pool }
    }
    pub async fn list(&self) -> ResponseResult<Vec<CloudMeta>> {
        let row: Vec<CloudMeta> = sqlx::query_as("select * from cloud_meta where deleted = ? ")
            .bind(i8::from(false))
            .fetch_all(&self.db_pool).await?;
        Ok(row)

        // let result = CloudMeta::select_by_column(&self.batis.clone(), "deleted", i8::from(false)).await;
        // let result = match result {
        //     Ok(vec) => Ok(vec),
        //     Err(e) => Err(e)
        // };
        // Ok(result?)
    }
    pub async fn select_by_status(&self, status: i8) -> ResponseResult<Vec<CloudMeta>> {
        let row: Vec<CloudMeta> = sqlx::query_as("select * from cloud_meta where status = ? and deleted = 0 ")
            .bind(status)
            .fetch_all(&self.db_pool).await?;
        Ok(row)

        // Ok(CloudMeta::select_by_column(&self.batis.clone(), "status", status)
        //     .await?)
    }
    pub async fn quota_random(&self, status: i8, size: i32) -> ResponseResult<Vec<CloudMeta>> {
        let row: Vec<CloudMeta> = sqlx::query_as("select * from cloud_meta where status = ? and deleted = 0 order by remaining_quota * random() desc limit ? ")
            .bind(status)
            .bind(size)
            .fetch_all(&self.db_pool).await?;
        Ok(row)
    }
    pub async fn query_token_timeout_cloud_meta(&self, now: u64) -> ResponseResult<Vec<CloudMeta>> {
        let row: Vec<CloudMeta> = sqlx::query_as("select * from cloud_meta where status = ? and deleted = 0 and expires_in < ?")
            .bind::<i8>(MetaStatus::Enable.into())
            .bind::<i64>(now as i64)
            .fetch_all(&self.db_pool).await?;
        Ok(row)
        // Ok(CloudMeta::query_token_timeout(&self.batis.clone(), MetaStatus::Enable.into(), now).await?)
    }
    pub async fn update(&self, meta: &CloudMeta) -> ResponseResult<u64> {
        let id= &meta.id;
        let name= &meta.name;
        let auth= &meta.auth;
        let last_work_time= &meta.last_work_time;
        let data_root= &meta.data_root;
        let status= &meta.status;
        let deleted= &meta.deleted;
        let cloud_type= &meta.cloud_type;
        let total_quota= &meta.total_quota;
        let used_quota= &meta.used_quota;
        let remaining_quota= &meta.remaining_quota;
        let extra= &meta.extra;
        let expires_in= &meta.expires_in;
        let result = sqlx::query!(r#"
        update cloud_meta set
        name =?, auth =?, last_work_time =?,
        data_root =?, status =?, deleted =?,
        cloud_type =?, total_quota =?,
        used_quota =?, remaining_quota =?, extra =?, expires_in =?
        where id  =?
        "#,
            name,
            auth,
            last_work_time,
            data_root,
            status,
            deleted,
            cloud_type,
            total_quota,
            used_quota,
            remaining_quota,
            extra,
            expires_in,
            id)
            .execute(&self.db_pool).await?;

        Ok(result.rows_affected())
    }
    pub async fn insert(&self, meta: &CloudMeta) -> ResponseResult<i64> {
        let name= &meta.name;
        let auth= &meta.auth;
        let last_work_time= &meta.last_work_time;
        let data_root= &meta.data_root;
        let status= &meta.status;
        let deleted= &meta.deleted;
        let cloud_type= &meta.cloud_type;
        let total_quota= &meta.total_quota;
        let used_quota= &meta.used_quota;
        let remaining_quota= &meta.remaining_quota;
        let extra= &meta.extra;
        let expires_in= &meta.expires_in;
        let result = sqlx::query!(r#"
        insert into cloud_meta (name, auth, last_work_time, data_root, status, deleted, cloud_type, total_quota, used_quota, remaining_quota, extra, expires_in) values (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
        "#,name,
            auth,
            last_work_time,
            data_root,
            status,
            deleted,
            cloud_type,
            total_quota,
            used_quota,
            remaining_quota,
            extra,
            expires_in)
            .execute(&self.db_pool).await?;
        Ok(result.last_insert_rowid())
    }
}

impl CloudMetaManager for SimpleCloudMetaManager {
    async fn add(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        let mut meta = meta.clone();
        meta.deleted = i8::from(false);
        let id = self.insert(&meta).await?;
        self.info(id as i32).await
    }

    async fn info(&self, id: i32) -> ResponseResult<CloudMeta> {
        let vec = sqlx::query_as("select * from cloud_meta where id =?")
            .bind(id)
            .fetch_all(&self.db_pool).await;
        // let vec = CloudMeta::select_by_column(&self.batis.clone(), "id", id)
        //     .await;
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
        // CloudMeta::update_by_column(&self.batis.clone(), meta, "id")
        //     .await
        //     ?;
        self.update(&meta).await?;

        self.info(meta.id.unwrap()).await
    }

    async fn delete(&self, id: i32) -> ResponseResult<CloudMeta> {
        let mut meta = self.info(id).await?;
        meta.deleted = i8::from(true);
        self.update(&meta).await?;
        self.info(id).await
    }
}
