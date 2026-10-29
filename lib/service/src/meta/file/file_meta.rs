use crate::meta::FileMetaType;
use crate::meta::FileMetaType::DIR;
use crate::DbPool;
use api::error::ErrorInfo;
use api::util::IntoOne;
use api::ResponseResult;
use async_recursion::async_recursion;
use chrono::Local;
use persistence::meta::FileMeta;
use persistence::FileStatus;
use std::ops::Add;
use log::error;

#[derive(Clone)]
pub struct SimpleFileMetaManager {
    db_pool: DbPool,
}

impl SimpleFileMetaManager {}

impl SimpleFileMetaManager {
    pub fn new(db_pool: DbPool) -> Self {
        SimpleFileMetaManager { db_pool }
    }

    pub async fn delete_meta(&self, meta: &FileMeta) -> ResponseResult<u64> {
        let result = sqlx::query("delete from file_meta where id = ?").bind(meta.id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
    pub async fn update_status(&self, file_meta: &FileMeta, target_status: FileStatus) -> ResponseResult<u64> {
        let x = sqlx::query("update file_meta set status=? where id =? and status =?")
            .bind::<i8>(target_status.into())
            .bind(&file_meta.id)
            .bind(&file_meta.status)
            .execute(&self.db_pool).await?;
        Ok(x.rows_affected())
    }
    pub async fn update_meta(&self, meta: &FileMeta) -> ResponseResult<FileMeta> {
        self.update(&meta).await?;

        // FileMeta::update_by_column(&self.batis.clone(), meta, "id")
        //     .await
        //     ?;
        self.info_by_id(meta.id.unwrap().into()).await
    }

    pub(crate) async fn info_by_id(&self, id: i64) -> ResponseResult<FileMeta> {
        let vec = sqlx::query_as("select * from file_meta where id=?")
            .bind(id)
            .fetch_all(&self.db_pool)
            .await;
        let vec = match vec {
            Ok(v) => v,
            Err(e) => {
                error!("{}",e);
                return Err(e.into());
            }
        };
        // let vec = FileMeta::select_by_column(&self.batis.clone(), "id", id).await?;
        if vec.is_empty() {
            Err(ErrorInfo::new(7000, &format!("文件{id}")))
        } else {
            Ok(vec.into_one().unwrap())
        }
    }

    pub(crate) async fn list_by_parent(&self, parent_id: i64) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where parent_id=?")
            .bind(parent_id)
            .fetch_all(&self.db_pool).await?;
        Ok(vec)
        // Ok(FileMeta::select_by_parent(&self.batis.clone(), parent_id)
        //     .await?)
    }

    pub(crate) async fn info_by_parent_and_name(
        &self,
        parent_id: i64,
        name: &str,
    ) -> ResponseResult<FileMeta> {
        let vec = sqlx::query_as("select * from file_meta where parent_id=? and name = ?")
            .bind(parent_id)
            .bind(name)
            .fetch_all(&self.db_pool)
            .await?;
        if vec.is_empty() {
            Err(ErrorInfo::new(7000, &format!("文件{name}")))
        } else {
            Ok(vec.into_one().unwrap())
        }
        // return if vec.is_empty() { None } else { vec.into_one() };
    }
    #[async_recursion]
    pub(crate) async fn delete_file_meta(&self, id: i64) -> ResponseResult<FileMeta> {
        // info!("delete file:{}", id);
        let file_meta = self.info_by_id(id).await?;

        let file_meta_type = file_meta.clone().file_type;
        if file_meta_type == DIR.get_code() {
            let vec = self.list_by_parent(file_meta.id.unwrap().into()).await?;
            for file_me in vec {
                let _result1 = self.delete_file_meta(file_me.id.unwrap().into()).await?;
            }
        }
        let mut file_meta = file_meta.clone();
        file_meta.deleted = 1;
        file_meta.update_time = Local::now();
        self.update(&file_meta).await?;
        // FileMeta::update_by_column(&self.batis.clone(), &file_meta, "id")
        //     .await?;

        Ok(file_meta)
    }

    pub(crate) async fn delete_one_file_meta(&self, id: i64) -> ResponseResult<FileMeta> {
        // info!("delete file:{}", id);
        let file_meta = self.info_by_id(id).await?;
        let mut file_meta = file_meta.clone();
        file_meta.deleted = 1;
        file_meta.update_time = Local::now();
        self.update(&file_meta).await?;
        // FileMeta::update_by_column(&self.batis.clone(), &file_meta, "id")
        //     .await?;

        Ok(file_meta)
    }
    pub(crate) async fn new_file(
        &self,
        parent_id: i64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<i64> {
        let value = FileMeta {
            id: None,
            name: name.to_string(),
            parent_id,
            file_type: file_type.get_code(),
            mode: 0o644,
            gid: 1000,
            uid: 1000,
            file_length: 0,
            status: FileStatus::Init.into(),
            deleted: 0,
            create_time: Local::now(),
            update_time: Local::now(),
        };
        Ok(self.insert(&value).await?)
        // Ok(FileMeta::insert(&self.batis.clone(), &value)
        //     .await
        //     ?
        //     .rows_affected)
    }
    async fn update(&self, meta: &FileMeta) -> ResponseResult<u64> {
        let sql = String::from("update file_meta set");
        let sql = sql.add(" name = ?");
        let sql = sql.add(",parent_id = ?");
        let sql = sql.add(",file_type = ?");
        let sql = sql.add(",mode = ?");
        let sql = sql.add(",gid = ?");
        let sql = sql.add(",uid = ?");
        let sql = sql.add(",file_length = ?");
        let sql = sql.add(",status = ?");
        let sql = sql.add(",deleted = ?");
        let sql = sql.add(",create_time = ?");
        let sql = sql.add(",update_time = ?");
        let sql = sql.add(" where id = ?");
        let result = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(&meta.name)
            .bind(&meta.parent_id)
            .bind(&meta.file_type)
            .bind(&meta.mode)
            .bind(&meta.gid)
            .bind(&meta.uid)
            .bind(&meta.file_length)
            .bind(&meta.status)
            .bind(&meta.deleted)
            .bind(&meta.create_time)
            .bind(&meta.update_time)
            .bind(&meta.id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
    async fn insert(&self, meta: &FileMeta) -> ResponseResult<i64> {
        let sql = String::from("insert into file_meta (name,parent_id,file_type,mode,gid,uid,file_length,status,deleted,create_time,update_time) values(?,?,?,?,?,?,?,?,?,?,?)");
        let result = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(&meta.name)
            .bind(&meta.parent_id)
            .bind(&meta.file_type)
            .bind(&meta.mode)
            .bind(&meta.gid)
            .bind(&meta.uid)
            .bind(&meta.file_length)
            .bind(&meta.status)
            .bind(&meta.deleted)
            .bind(&meta.create_time)
            .bind(&meta.update_time)
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
}
