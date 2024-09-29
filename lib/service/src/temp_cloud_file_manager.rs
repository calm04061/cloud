use api::ResponseResult;
use persistence::{FileStatus, TempCloudFile};
use crate::DbPool;

pub struct TempCloudFileManager {
    db_pool: DbPool,
}

impl TempCloudFileManager {
    pub fn new(db_pool: DbPool) -> TempCloudFileManager {
        TempCloudFileManager {
            db_pool,
        }
    }
    async fn find_by_status(&self, status: FileStatus) -> ResponseResult<Vec<TempCloudFile>> {
        let vec = sqlx::query_as("select * from temp_cloud_file where status = ?")
            .bind::<i8>(status.into())
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }

    pub async fn add(&self, file: &TempCloudFile) -> ResponseResult<i64> {
        let result = sqlx::query("insert into temp_cloud_file (cloud_meta_id,cloud_meta_id,status,create_time,update_time) values (?,?,?,?,?,?)")
            .bind(&file.cloud_file_id)
            .bind(&file.cloud_meta_id)
            .bind(&file.status)
            .bind(&file.create_time)
            .bind(&file.update_time)
            .execute(&self.db_pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    async fn update_status(&self, file: TempCloudFile, status: FileStatus) -> ResponseResult<u64> {
        let result = sqlx::query("update temp_cloud_file set status = ? where id=? and status = ?")
            .bind::<i8>(status.into())
            .bind(file.id)
            .bind(file.status)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
    async fn delete_by_id(&self, id: i32) -> ResponseResult<u64> {
        let result = sqlx::query("delete from temp_cloud_file  where id=? ")
            .bind(id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
}