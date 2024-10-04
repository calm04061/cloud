use crate::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::meta::file::file_meta::SimpleFileMetaManager;
use crate::meta::{FileManager, FileMetaType};
use crate::DbPool;
use api::error::ErrorInfo;
use api::util::IntoOne;
use api::ResponseResult;
use chrono::{DateTime, Utc};
use log::error;
use persistence::meta::{FileBlockMeta, FileMeta};

pub mod file_block_meta;
pub mod file_meta;

pub struct SimpleFileManager {
    db_pool: DbPool,
    file_meta_manager: SimpleFileMetaManager,
    file_block_meta_manager: SimpleFileBlockMetaManager,
}

impl SimpleFileManager {
    pub fn new(db_pool: DbPool, file_meta_manager: SimpleFileMetaManager, file_block_meta_manager: SimpleFileBlockMetaManager) -> Self {
        SimpleFileManager { db_pool, file_meta_manager, file_block_meta_manager }
    }

    pub async fn delete_file_meta(&self, id: i64) -> ResponseResult<FileMeta> {
        self
            .file_meta_manager
            .delete_file_meta(id)
            .await
    }
    async fn select_by_parent_page(&self, parent_id: i64, start: i64, size: usize) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where parent_id = #{parent_id} and id> #{start} and deleted = 0 order by id  limit #{size}")
            .bind(parent_id)
            .bind(start)
            .bind(size as i32)
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }
    pub async fn list_all_by_parent(&self, parent_id: i64) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where parent_id = ?")
            .bind(parent_id)
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }
}

impl FileManager for SimpleFileManager {
    async fn list_deleted_file(&self, update_time: DateTime<Utc>, pre_id: i64) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where id> ? and update_time <= ? and deleted = 1 and status <> 7 order by id limit 30")
            .bind(pre_id)
            .bind(update_time)
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }

    async fn list_by_parent(&self, parent_id: i64) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where parent_id = ? and deleted = ?")
            .bind(parent_id)
            .bind(0)
            .fetch_all(&self.db_pool)
            .await?;
        Ok(vec)
    }
    async fn list_by_parent_page(&self, parent_id: i64, start: i64, size: usize) -> ResponseResult<(Vec<FileMeta>, bool)> {
        let vec = self.select_by_parent_page(parent_id, start, size).await?;
        if vec.is_empty() {
            return Ok((vec, true));
        }
        let last = vec.get(vec.len() - 1).unwrap().id.unwrap();

        let more = self.select_by_parent_page(parent_id, last, 1).await?;
        Ok((vec, more.is_empty()))
    }

    async fn info_by_parent_and_name(&self, parent_id: i64, name: &str) -> ResponseResult<FileMeta> {
        let vec = sqlx::query_as("select * from file_meta where parent_id=? and name = ? and deleted = 0")
            .bind(parent_id)
            .bind(name)
            .fetch_all(&self.db_pool)
            .await?;
        if vec.is_empty() {
            Err(ErrorInfo::new(12, "文件不存在"))
        } else {
            let one = vec.into_one().unwrap();
            Ok(one)
        }
    }

    async fn new_file(
        &self,
        parent_id: i64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta> {
        self.file_meta_manager.new_file(
            parent_id, name, file_type.clone())
            .await?;
        let result = self.
            file_meta_manager
            .info_by_parent_and_name(parent_id, name)
            .await;
        if let Err(e) = result {
            error!("{}", e);
            return Err(ErrorInfo::new(12, "创文件失败"));
        }
        let f = result?;
        Ok(f)
    }

    async fn update_meta(&self, meta: FileMeta) -> ResponseResult<FileMeta> {
        self.file_meta_manager.update_meta(&meta).await
    }

    async fn update_file_content(&self, meta: FileMeta, block_index: usize) -> ResponseResult<FileMeta> {
        let file_meta = self.file_meta_manager.update_meta(&meta).await?;

        let result = self
            .file_block_meta_manager
            .delete_file_blocks(meta.id.unwrap(), block_index as i64)
            .await;
        match result {
            Ok(_v) => {}
            Err(e) => {
                error!("{}", e);
            }
        };
        Ok(file_meta)
    }
    async fn delete_file_blocks(&self, file_id: i64, block_index: i64) -> ResponseResult<u64> {
        self.file_block_meta_manager
            .delete_file_blocks(file_id, block_index)
            .await
    }

    async fn info_by_id(&self, id: i64) -> ResponseResult<FileMeta> {
        self.file_meta_manager.info_by_id(id).await
    }


    async fn delete_one_file_meta(&self, id: i64) -> ResponseResult<FileMeta> {
        self
            .file_meta_manager
            .delete_one_file_meta(id)
            .await
    }
    async fn clean_file_meta(&self, id: i64) -> ResponseResult<Option<FileMeta>> {
        let file_meta = self.file_meta_manager.info_by_id(id).await?;
        self
            .file_block_meta_manager
            .delete_file_meta_block_by_file_meta_id(id)
            .await
            ?;
        self
            .file_meta_manager
            .delete_meta(&file_meta)
            .await
            ?;
        Ok(Some(file_meta))
    }
    async fn file_block_meta(&self, file_meta_id: i64) -> ResponseResult<Vec<FileBlockMeta>> {
        self
            .file_block_meta_manager
            .file_block_meta(file_meta_id)
            .await
    }

    async fn file_block_meta_index(
        &self,
        file_meta_id: i64,
        block_index: i64,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        self
            .file_block_meta_manager
            .file_block_meta_index(file_meta_id, block_index)
            .await
    }

    async fn file_block_meta_info_by_id(&self, id: i32) -> ResponseResult<Option<FileBlockMeta>> {
        self
            .file_block_meta_manager
            .file_block_meta_info_by_id(id)
            .await
    }

    async fn update_file_block_meta(
        &self,
        mut meta: FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        let option = self
            .file_block_meta_manager
            .update_file_block_meta(&mut meta)
            .await;
        option
    }

    async fn save_file_block_meta(&self, meta: FileBlockMeta) -> ResponseResult<Option<FileBlockMeta>> {
        let option = self
            .file_block_meta_manager
            .save_file_block_meta(meta)
            .await;
        option
    }

    async fn modified_blocks(&self, before: i64) -> ResponseResult<Vec<FileBlockMeta>> {
        let vec = self
            .file_block_meta_manager
            .modified_blocks(before)
            .await;
        vec
    }
}
