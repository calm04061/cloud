use chrono::{DateTime, Utc};
use log::error;
use crate::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::meta::file::file_meta::SimpleFileMetaManager;
use crate::meta::{FileManager, FileMetaType};
use api::error::ErrorInfo;
use api::util::IntoOne;
use api::ResponseResult;
use persistence::meta::{FileBlockMeta, FileMeta};
use persistence::{FileStatus};
use crate::cloud_file_block_manager::CloudFileBlockManager;
use crate::DbPool;

pub mod file_block_meta;
pub mod file_meta;

pub struct SimpleFileManager {
    // batis: RBatis,
    db_pool: DbPool,
    file_meta_manager: SimpleFileMetaManager,
    file_block_meta_manager: SimpleFileBlockMetaManager,
    cloud_file_block_manager: CloudFileBlockManager,
}

impl SimpleFileManager {
    pub fn new(db_pool: DbPool, file_meta_manager: SimpleFileMetaManager, file_block_meta_manager: SimpleFileBlockMetaManager, cloud_file_block_manager: CloudFileBlockManager) -> Self {
        SimpleFileManager { db_pool, file_meta_manager, file_block_meta_manager, cloud_file_block_manager }
    }
    pub async fn mark_clean_file_block(&self, meta: &mut FileMeta) -> ResponseResult<()> {
        // let batis = self.batis.clone();
        let file_block_metas = self.file_block_meta_manager.file_block_meta(meta.id.unwrap()).await?;
        // let file_block_metas = FileBlockMeta::select_by_file_meta_id(&batis, meta.id.unwrap()).await?;
        // let mut tran = batis.acquire_begin().await?;
        for mut file_block_meta in file_block_metas {
            let cloud_file_blocks = self.cloud_file_block_manager.select_by_file_block_id(file_block_meta.id.unwrap()).await?;
            // let cloud_file_blocks = CloudFileBlock::select_by_file_block_id(&tran, file_block_meta.id.unwrap()).await?;
            for mut cloud_file_block in cloud_file_blocks {
                let status: FileStatus = cloud_file_block.status.try_into()?;
                if status == FileStatus::Cleaning {
                    continue;
                }
                if cloud_file_block.deleted == 1 && status == FileStatus::Cleaned {
                    continue;
                }
                cloud_file_block.status = FileStatus::WaitClean.into();
                cloud_file_block.deleted = 1;
                self.cloud_file_block_manager.update(&cloud_file_block).await?;
                // CloudFileBlock::update_by_column(&tran, &cloud_file_block, "id").await?;
            }
            let status: FileStatus = file_block_meta.status.try_into()?;
            if status == FileStatus::Cleaning {
                continue;
            }
            if file_block_meta.deleted == 1 && status == FileStatus::Cleaned {
                continue;
            }
            if file_block_meta.deleted != 1 || status != FileStatus::WaitClean {
                file_block_meta.deleted = 1;
                file_block_meta.status = FileStatus::WaitClean.into();
                self.file_block_meta_manager.update(&file_block_meta).await?;
                // FileBlockMeta::update_by_column(&tran, &file_block_meta, "id").await?;
            }
        }

        let status: FileStatus = meta.status.try_into()?;
        if status == FileStatus::Cleaning {
            // tran.commit().await?;
            return Ok(());
        }
        if meta.deleted == 1 && status == FileStatus::Cleaned {
            // tran.commit().await?;
            return Ok(());
        }
        if meta.deleted != 1 || status != FileStatus::WaitClean {
            meta.deleted = 1;
            meta.status = FileStatus::WaitClean.into();
            self.file_meta_manager.update_meta(meta).await?;
            // FileMeta::update_by_column(&tran, &meta, "id").await?;
        }
        // tran.commit().await?;
        Ok(())
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
}

impl FileManager for SimpleFileManager {
    async fn list_deleted_file(&self, update_time: DateTime<Utc>) -> ResponseResult<Vec<FileMeta>> {
        let vec = sqlx::query_as("select * from file_meta where update_time <= ? and file_type in (1,3) and deleted = 1 and status <> 7 order by update_time desc limit 1000")
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
        // let vec = FileMeta::select_by_parent_page(&self.batis.clone(), parent_id, start, size)
        //     .await?;
        if vec.is_empty() {
            return Ok((vec, true));
        }
        let last = vec.get(vec.len() - 1).unwrap().id.unwrap();

        let more = self.select_by_parent_page(parent_id, last, 1).await?;
        // FileMeta::select_by_parent_page(&self.batis.clone(), parent_id, last, 1)
        // .await?;

        Ok((vec, more.is_empty()))
    }

    async fn info_by_parent_and_name(&self, parent_id: i64, name: &str) -> ResponseResult<FileMeta> {
        let vec = sqlx::query_as("select * from file_meta where parent_id=? and name = ? and deleted = 0")
            .bind(parent_id)
            .bind(name)
            .fetch_all(&self.db_pool)
            .await?;
        // let vec = FileMeta::info_by_parent_and_name(&self.batis.clone(), parent_id, name)
        //     .await?;
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
    async fn delete_file_blocks(&self, file_id: i64, block_index: i64) -> () {
        let result = self
            .file_block_meta_manager
            .delete_file_blocks(file_id, block_index)
            .await;
        match result {
            Ok(_) => {}
            Err(_) => {}
        }
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
