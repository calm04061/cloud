use std::cell::Cell;
use chrono::Utc;
use crate::{DbPool, CONTEXT};
use api::util::IntoOne;
use api::ResponseResult;
use log::debug;
use sqlx::Transaction;
use sqlx_sqlite::Sqlite;
use persistence::meta::{CloudMeta, FileBlockMeta};
use persistence::{CloudFileBlock, FileStatus, MetaStatus};
thread_local!(static TRANSACTION: Cell<Option<Transaction<'static,Sqlite>> > = Cell::new(None));

#[derive(Clone)]
pub struct SimpleFileBlockMetaManager {
    db_pool: DbPool,
}

impl SimpleFileBlockMetaManager {
    pub fn new(db_pool: DbPool) -> Self {
        SimpleFileBlockMetaManager { db_pool }
    }
    pub(crate) async fn file_block_meta_info_by_id(&self, id: i32) -> ResponseResult<Option<FileBlockMeta>> {
        let rows = sqlx::query_as("select * from file_block_meta where id =? ")
            .bind(id)
            .fetch_all(&self.db_pool)
            .await?
            ;
        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(rows.into_one())
        }
    }
    pub async fn update_file_block_meta(
        &self,
        meta: &mut FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        meta.update_time = Utc::now();
        // FileBlockMeta::update_by_column(&self.batis.clone(), &meta, "id")
        //     .await
        //     ?;
        self.update(&meta).await?;
        self.file_block_meta_info_by_id(meta.id.unwrap()).await
    }
    pub async fn insert(&self, meta: &FileBlockMeta) -> ResponseResult<i64> {
        //         pub id: Option<i32>,
        //         pub block_index: i64,
        //         pub file_part_id: String,
        //         pub update_time: i64,
        //         pub file_modify_time: i64,
        //         pub deleted: i8,
        //         pub file_meta_id: i64,
        //         pub part_hash: String,
        //         pub status: i8,

        let result = sqlx::query!("insert into file_block_meta (id, block_index, file_part_id, update_time,file_modify_time,deleted,file_meta_id,part_hash,status) values(?,?,?,?,?,?,?,?,?) "
            ,meta.id,meta.block_index,meta.file_part_id,meta.update_time,meta.file_modify_time,meta.deleted,meta.file_meta_id,meta.part_hash,meta.status)
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }
    pub async fn update(&self, meta: &FileBlockMeta) -> ResponseResult<u64> {
        let sql = self.update_field_sql();
        let sql = sql + "  where id=?";
        let query = sqlx::query(sql.as_str());
        Ok(query
            .bind(&meta.block_index)
            .bind(&meta.file_part_id)
            .bind(&meta.update_time)
            .bind(&meta.file_modify_time)
            .bind(&meta.deleted)
            .bind(&meta.file_meta_id)
            .bind(&meta.part_hash)
            .bind(&meta.status)
            .bind(&meta.id)
            .execute(&self.db_pool)
            .await?.rows_affected())
    }
    pub(crate) async fn save_file_block_meta(
        &self,
        mut meta: FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        if meta.id.is_none() {
            self.insert(&meta).await?;
            // FileBlockMeta::insert(&self.batis.clone(), &meta).await?;
            let option = self.file_block_meta_index(meta.file_meta_id, meta.block_index).await?;
            let meta = option.unwrap();

            let file_block_meta_id = meta.id.unwrap();
            let vec: Vec<CloudMeta> = sqlx::query_as("select * from cloud_meta where status = ? and deleted = 0 order by remaining_quota * random() desc limit ?")
                .bind::<i8>(MetaStatus::Enable.into())
                .bind(3)
                .fetch_all(&self.db_pool)
                .await?;
            // let vec = CloudMeta::quota_random(&self.batis.clone(), MetaStatus::Enable.into(), 3)
            //     .await?;
            for cloud in vec {
                let block = CloudFileBlock::init(file_block_meta_id, cloud.id.unwrap());
                CONTEXT.cloud_file_block_manager.insert(&block).await?;
                // CloudFileBlock::insert(&self.batis.clone(), &block).await?;
            }
            Ok(Some(meta))
        } else {
            meta.update_time = Utc::now();
            self.update(&meta).await?;
            self.file_block_meta_info_by_id(meta.id.unwrap()).await
        }
    }

    pub(crate) async fn file_block_meta_index(
        &self,
        file_meta_id: i64,
        block_index: i64,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        let sql = "select * from file_block_meta where deleted = 0 and file_meta_id = ? and block_index = ?";
        let vec = sqlx::query_as(sql)
            .bind(file_meta_id)
            .bind(block_index)
            .fetch_all(&self.db_pool)
            .await?;
        if vec.is_empty() {
            Ok(None)
        } else {
            Ok(vec.into_one())
        }
    }

    pub(crate) async fn file_block_meta(&self, file_meta_id: i64) -> ResponseResult<Vec<FileBlockMeta>> {
        Ok(sqlx::query_as("select * from file_block_meta where deleted = 0 and file_meta_id =?")
            .bind(file_meta_id)
            .fetch_all(&self.db_pool)
            .await?)
    }

    pub(crate) async fn modified_blocks(&self, _before: i64) -> ResponseResult<Vec<FileBlockMeta>> {
        let vec = sqlx::query_as("select * from file_block_meta where (part_hash <> cloud_file_hash or cloud_file_hash is null ) and deleted = 0 order by update_time,id")
            .fetch_all(&self.db_pool).await?;
        Ok(vec)
    }

    pub(crate) async fn delete_file_blocks(
        &self,
        file_id: i64,
        block_index: i64,
    ) -> ResponseResult<u64> {
        Ok(sqlx::query("update file_block_meta set deleted = 1,update_time=? where file_meta_id=? and block_index>? and deleted = 0")
            .bind(chrono::Local::now().timestamp_millis())
            .bind(file_id)
            .bind(block_index)
            .execute(&self.db_pool)
            .await?.rows_affected())


        // let batis = self.batis.clone();
        // Ok(batis.exec("update file_block_meta set deleted = 1,update_time=? where file_meta_id=? and block_index>? and deleted = 0", vec![to_value!(chrono::Local::now().timestamp_millis()), to_value!(file_id), to_value!(block_index)]).await?.rows_affected)
    }
    pub async fn clean_file_meta(&self) ->ResponseResult<u64>{
        let result = sqlx::query(r#"
            delete from file_block_meta where id in(
            select t1.id from (
         select fbm.id,count(cfb.id) c from file_block_meta fbm left join cloud_file_block cfb on fbm.id = cfb.file_block_id where fbm.deleted = 1 and fbm.status=9
         group by fbm.id
                       ) as t1 where t1.c = 0
                       )
        "#).execute(&self.db_pool).await?;
        Ok(result.rows_affected())
    }

    pub(crate) async fn delete_file_meta_block_by_file_meta_id(
        &self,
        file_meta_id: i64,
    ) -> ResponseResult<u64> {
        let vec: Vec<FileBlockMeta> = sqlx::query_as("select * from file_block_meta where deleted = 0 and file_meta_id=?")
            .bind(file_meta_id)
            .fetch_all(&self.db_pool)
            .await?;
        for meta in vec {
            sqlx::query!("delete from cloud_file_block where cloud_meta_id=?", meta.id)
                .execute(&self.db_pool).await?;
        }
        debug!("delete block meta by file id:{}", file_meta_id);
        let result = sqlx::query!("delete from file_block_meta where deleted =0 and file_meta_id=?",file_meta_id).execute(&self.db_pool).await?;
        Ok(
            result.rows_affected()
        )
    }
    pub async fn update_by_status(&self, block: &FileBlockMeta, status: FileStatus) -> ResponseResult<u64> {
        let sql = self.update_field_sql();
        let sql = sql + " where id=? and status = ?";
        let result = sqlx::query(sql.as_str())
            .bind(&block.block_index)
            .bind(&block.file_part_id)
            .bind(&block.update_time)
            .bind(&block.file_modify_time)
            .bind(&block.deleted)
            .bind(&block.file_meta_id)
            .bind(&block.part_hash)
            .bind(&block.status)
            .bind(&block.id)
            .bind::<i8>(status.into())
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())

        // Ok(FileBlockMeta::update_by_status(&self.batis.clone(), block, block.id.unwrap(), status.into()).await?.rows_affected)
    }
    pub async fn select_by_status_limit(&self, status: FileStatus, size: usize) -> ResponseResult<Vec<FileBlockMeta>> {
        Ok(sqlx::query_as("select * from file_block_meta where status=? limit ?")
            .bind::<i8>(status.into())
            .bind::<i32>(size as i32)
            .fetch_all(&self.db_pool)
            .await?)
        // Ok(FileBlockMeta::select_by_status_limit(&self.batis.clone(), status.into(), size).await?)
    }
    fn update_field_sql(&self) -> String {
        let sql = String::from("update file_block_meta set ");
        let sql = sql + " block_index=? ";
        let sql = sql + ",file_part_id = ? ";
        let sql = sql + ",update_time=? ";
        let sql = sql + ",file_modify_time=? ";
        let sql = sql + ",deleted=? ";
        let sql = sql + ",file_meta_id=? ";
        let sql = sql + ",part_hash=? ";
        let sql = sql + ",status=? ";
        sql
    }
}
