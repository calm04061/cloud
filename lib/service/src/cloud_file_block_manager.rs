use crate::DbPool;
use api::ResponseResult;
use chrono::Local;
use persistence::{CloudFileBlock, FileStatus};
use sqlx::query;
use std::ops::Add;
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

    pub async fn update_status(&self, block: &CloudFileBlock, target_status: FileStatus) -> ResponseResult<u64> {
        let result = query("update cloud_file_block set status = ? where id = ? and status = ?")
            .bind::<i8>(target_status.into())
            .bind(&block.id.unwrap())
            .bind(&block.status)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn update_by_status(&self, block: &CloudFileBlock, status: FileStatus) -> ResponseResult<u64> {
        let sql = String::from("update cloud_file_block set ");
        let sql = sql.add(" cloud_meta_id =?,\
            cloud_file_hash =?,\
            cloud_file_id =?,\
            status =?,\
            deleted =?,\
            create_time =?,\
            update_time =?,\
            retry_count =?,\
            next_retry_time =?");
        let sql = sql.add(" where id=? and status=?");

        let result = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.update_time)
            .bind(&block.retry_count)
            .bind(&block.next_retry_time)
            .bind(&block.id)
            .bind::<i8>(status.into())
            .execute(&self.db_pool).await?;
        Ok(result.rows_affected())
    }
    pub async fn select_by_file_block_id(&self, file_block_id: i32) -> ResponseResult<Vec<CloudFileBlock>> {
        let vec = sqlx::query_as("select * from cloud_file_block where file_block_id =? order by cloud_file_hash")
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
            update_time =?,\
            retry_count =?,\
            next_retry_time =?\
             where id=?";
        Ok(sqlx::query(sql)
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.update_time)
            .bind(&block.retry_count)
            .bind(&block.next_retry_time)
            .bind(&block.id)
            .execute(&self.db_pool)
            .await?.rows_affected())
    }
    pub async fn insert(&self, block: &CloudFileBlock) -> ResponseResult<i64> {
        let sql = String::from("insert into cloud_file_block (file_block_id, cloud_meta_id,cloud_file_hash, cloud_file_id, status, deleted, create_time, update_time, retry_count, next_retry_time) values (?,?,?,?,?,?,?,?,?,?)");
        Ok(sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(&block.file_block_id)
            .bind(&block.cloud_meta_id)
            .bind(&block.cloud_file_hash)
            .bind(&block.cloud_file_id)
            .bind(&block.status)
            .bind(&block.deleted)
            .bind(&block.create_time)
            .bind(&block.update_time)
            .bind(&block.retry_count)
            .bind(&block.next_retry_time)
            .execute(&self.db_pool)
            .await?.last_insert_rowid())
    }
    pub async fn delete_by_id(&self, id: i32) -> ResponseResult<u64> {
        Ok(sqlx::query("delete from cloud_file_block where id =?")
            .bind(id)
            .execute(&self.db_pool).await?.rows_affected())
    }
    /// 捞取待上传/待重试的副本块。
    ///
    /// - now：退避窗口基准（只处理 next_retry_time 已到期的行）
    ///
    /// 选取范围：
    /// - status=1 (Init)：新入队
    /// - status=4 (UploadFail)：退避到期后重试
    /// - status=3 (UploadSuccess) 且对应 file_block_meta 哈希已不匹配：部件内容已变化，重新上传收敛副本
    ///
    /// status=2 (Uploading) 的中断行不在这里回收，由 reset 任务超时重置回 Init，
    /// 保证抢占始终是不同状态间的 CAS，多实例不会重复处理同一行。
    /// 状态 10 (Dead) 为死信，永不自动重试。
    pub async fn select_to_upload(&self, now: chrono::DateTime<Local>) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(sqlx::query_as("select cfb.* from cloud_file_block cfb left join file_block_meta fbm on cfb.file_block_id = fbm.id and cfb.cloud_file_hash = fbm.part_hash where cfb.deleted = 0 and (cfb.status = 1 or cfb.status = 4 or (fbm.id is null and cfb.status = 3)) and (cfb.next_retry_time is null or cfb.next_retry_time <= ?) order by cfb.update_time limit 8")
            .bind(now)
            .fetch_all(&self.db_pool)
            .await?)
    }

    pub async fn select_by_status(&self, status: FileStatus, update_time: chrono::DateTime<Local>) -> ResponseResult<Vec<CloudFileBlock>> {
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
        select id as file_block_id,size from (
            select fbm.id, count(cfb.id) size from file_block_meta fbm
            left join cloud_file_block cfb on fbm.id = cfb.file_block_id
            left join file_meta fm on fbm.file_meta_id = fm.id
            where fm.deleted = 0
            group by fbm.id
        ) where size <> ? limit 500
        "#;
        Ok(sqlx::query_as(sql)
            .bind(size)
            .fetch_all(&self.db_pool)
            .await?)
    }

    /// 按状态统计未删除的副本块行数（/metrics 用）。
    pub async fn status_counts(&self) -> ResponseResult<Vec<(i8, i64)>> {
        let rows: Vec<(i8, i64)> = sqlx::query_as(
            "select status, count(*) from cloud_file_block where deleted = 0 group by status",
        )
        .fetch_all(&self.db_pool)
        .await?;
        Ok(rows)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct TempRow {
    pub file_block_id: Option<i32>,
    pub size: u32,
}
