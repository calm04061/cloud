use std::ops::Add;
use chrono::Utc;
use sqlx::{query};
use api::ResponseResult;
use persistence::{CloudFileBlock, FileStatus};
use crate::DbPool;
#[derive(Clone)]
pub struct CloudFileBlockManager {
    db_pool: DbPool,

}

impl CloudFileBlockManager {
    pub fn new(db_pool: DbPool) -> CloudFileBlockManager {
        CloudFileBlockManager {
            db_pool,
        }
    }
    pub async fn clean_deleted_block(&self) -> ResponseResult<u64> {
        let result = query("delete from cloud_file_block where deleted = ? and status = ?")
            .bind(1)
            .bind::<i8>(FileStatus::Cleaned.into())
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
    pub async fn update_by_status(&self, block: &CloudFileBlock, status: FileStatus) -> ResponseResult<u64> {
        let sql = String::from("update cloud_file_block set ");
        let sql = sql.add(" id = ? ,\
            cloud_meta_id =?,\
            cloud_file_hash =?,\
            cloud_file_id =?,\
            status =?,\
            deleted =?,\
            create_time =?,\
            update_time =?");
        let sql = sql.add(" where id=? and status=?");

        let result = sqlx::query(sql.as_str())
            .bind(&block.id)
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.update_time)
            .bind(&block.id)
            .bind::<i8>(status.into())
            .execute(&self.db_pool).await?;
        Ok(result.rows_affected())
        // Ok(CloudFileBlock::update_by_status(&self.batis.clone(), block, block.id.unwrap(), status.into()).await?.rows_affected)
    }
    pub async fn select_by_file_block_id(&self, file_block_id: i32) -> ResponseResult<Vec<CloudFileBlock>> {
        let vec = sqlx::query_as("select * from cloud_file_block where file_block_id =?")
            .bind(file_block_id)
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
        // Ok(CloudFileBlock::select_by_column(&self.batis.clone(), "file_block_id", file_block_id)
        //     .await?)
    }
    pub async fn update(&self, block: &CloudFileBlock) -> ResponseResult<u64> {
        let sql = "update cloud_file_block set \
            cloud_meta_id =?, \
            cloud_file_hash =?, \
            cloud_file_id =?, \
            status =?, \
            deleted =?,\
            create_time =?, \
            update_time =?\
             where id=?";
        Ok(sqlx::query(sql)
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.create_time)
            .bind(&block.id)
            .execute(&self.db_pool)
            .await?.rows_affected())
        // Ok(CloudFileBlock::update_by_column(&self.batis.clone(), block, "id")
        //     .await
        //     ?.rows_affected)
    }
    pub async fn insert(&self, block: &CloudFileBlock) -> ResponseResult<i64> {
        let sql = String::from("insert into cloud_file_block (file_block_id, cloud_meta_id,cloud_file_hash, cloud_file_id, status, deleted, create_time, update_time) values (?,?,?,?,?,?,?,?)");
        Ok(sqlx::query(sql.as_str())
            .bind(&block.file_block_id)
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.update_time)
            .execute(&self.db_pool)
            .await?.last_insert_rowid())
        // Ok(CloudFileBlock::insert(&self.batis.clone(), block).await?)
    }
    pub async fn delete_by_id(&self, id: i32) -> ResponseResult<u64> {
        Ok(sqlx::query("delete from cloud_file_block where id =?")
            .bind(id)
            .execute(&self.db_pool).await?.rows_affected())
    }
    pub async fn select_to_upload(&self) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(sqlx::query_as("select cfb.* from cloud_file_block cfb left join file_block_meta fbm on cfb.file_block_id = fbm.id and cfb.cloud_file_hash = fbm.part_hash where cfb.status = 1 or (fbm.id is null and cfb.status = 3) order by cfb.update_time limit 8")
            .fetch_all(&self.db_pool)
            .await?)
    }

    pub async fn select_by_status(&self, status: FileStatus, update_time: chrono::DateTime<Utc>) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(sqlx::query_as("select * from cloud_file_block where status =? and update_time <?")
            .bind::<i8>(status.into())
            .bind(update_time)
            .fetch_all(&self.db_pool)
            .await?)
    }
    pub async fn select_by_status_limit(&self, status: FileStatus, size: usize) -> ResponseResult<Vec<CloudFileBlock>> {
        //  where status=#{status} limit #{size}
        let row: Vec<CloudFileBlock> = sqlx::query_as("select * from cloud_file_block where status=? limit ?")
            .bind::<i8>(status.into())
            .bind::<i32>(size as i32)
            .fetch_all(&self.db_pool).await?;
        Ok(row)
    }
    pub async fn query_block_need_re_balance(&self, size: i32) -> ResponseResult<Vec<TempRow>> {

        // let rbatis = &self.batis.clone();
        let sql = r#"
        select id as file_block_id from (
            select fbm.id, count(cfb.id) size from file_block_meta fbm
            left join cloud_file_block cfb on fbm.id = cfb.file_block_id
            left join file_meta fm on fbm.file_meta_id = fm.id
            where fm.deleted = 0
            group by cfb.id
        ) where size < ? limit 50
        "#;
        Ok(sqlx::query_as(sql)
            .bind(size)
            .fetch_all(&self.db_pool)
            .await?)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct TempRow {
    pub file_block_id: Option<i32>,
}
