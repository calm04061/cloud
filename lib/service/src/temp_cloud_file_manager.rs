use crate::DbPool;
use api::ResponseResult;
use chrono::{DateTime, Local};
use persistence::{FileStatus, TempCloudFile};

pub struct TempCloudFileManager {
    db_pool: DbPool,
}

impl TempCloudFileManager {
    pub async fn select_by_status(&self, status: FileStatus, update_time: DateTime<Local>) -> ResponseResult<Vec<TempCloudFile>> {
        Ok(sqlx::query_as("select * from temp_cloud_file where status =? and update_time <?")
            .bind::<i8>(status.into())
            .bind(update_time)
            .fetch_all(&self.db_pool)
            .await?)
    }
}

impl TempCloudFileManager {
    pub fn new(db_pool: DbPool) -> TempCloudFileManager {
        TempCloudFileManager {
            db_pool,
        }
    }
    pub async fn find_by_status(&self, id: i64, status: FileStatus) -> ResponseResult<Vec<TempCloudFile>> {
        let vec = sqlx::query_as("select * from temp_cloud_file where id > ? and status = ? order by id")
            .bind(id)
            .bind::<i8>(status.into())
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }

    pub async fn add(&self, file: &TempCloudFile) -> ResponseResult<i64> {
        let result = sqlx::query("insert into temp_cloud_file (cloud_file_id,cloud_meta_id,status,create_time,update_time) values (?,?,?,?,?)")
            .bind(&file.cloud_file_id)
            .bind(&file.cloud_meta_id)
            .bind(&file.status)
            .bind(&file.create_time)
            .bind(&file.update_time)
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update_status(&self, file: &TempCloudFile, target_status: FileStatus) -> ResponseResult<u64> {
        let result = sqlx::query("update temp_cloud_file set status = ?,update_time =? where id=? and status = ?")
            .bind::<i8>(target_status.into())
            .bind(file.update_time)
            .bind(file.id.unwrap())
            .bind(file.status)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
    pub async fn delete_by_id(&self, id: i64) -> ResponseResult<u64> {
        let result = sqlx::query("delete from temp_cloud_file  where id=? ")
            .bind(id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
}