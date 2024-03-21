use log::error;
use rbatis::RBatis;

use api::error::ErrorInfo;
use api::ResponseResult;
use api::util::IntoOne;
use persistence::FileMeta;

use crate::database::meta::{FileBlockMeta, FileManager, FileMetaType};
use crate::database::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::database::meta::file::file_meta::SimpleFileMetaManager;

pub mod file_block_meta;
pub mod file_meta;

pub struct SimpleFileManager {
    batis: RBatis,
    file_meta_manager: SimpleFileMetaManager,
    file_block_meta_manager: SimpleFileBlockMetaManager,
}

impl SimpleFileManager {
    pub fn new(batis: RBatis, file_meta_manager: SimpleFileMetaManager, file_block_meta_manager: SimpleFileBlockMetaManager) -> Self {
        SimpleFileManager { batis, file_meta_manager, file_block_meta_manager }
    }
}

impl FileManager for SimpleFileManager {
    async fn list_deleted_file(&self, update_time: i64) -> ResponseResult<Vec<FileMeta>> {
        Ok(FileMeta::list_deleted_file(&self.batis.clone(), update_time).await.unwrap())
    }

    async fn list_by_parent(&self, parent_id: u64) -> ResponseResult<Vec<FileMeta>> {
        Ok(FileMeta::select_by_parent(&self.batis.clone(), parent_id)
            .await
            .unwrap())
    }
    async fn list_by_parent_page(&self, parent_id: u64, start: u64, size: usize) -> ResponseResult<(Vec<FileMeta>, bool)> {
        let vec = FileMeta::select_by_parent_page(&self.batis.clone(), parent_id, start, size)
            .await.unwrap();
        if vec.is_empty() {
            return Ok((vec, true));
        }
        let last = vec.get(vec.len() - 1).unwrap().id.unwrap();
        let more = FileMeta::select_by_parent_page(&self.batis.clone(), parent_id, last, 1)
            .await.unwrap();

        Ok((vec, more.is_empty()))
    }
    async fn info_by_parent_and_name(&self, parent_id: u64, name: &str) -> ResponseResult<FileMeta> {
        let vec = FileMeta::info_by_parent_and_name(&self.batis.clone(), parent_id, name)
            .await
            .unwrap();
        return if vec.is_empty() {
            return Err(ErrorInfo::new(12, "文件不存在"));
        } else {
            let one = vec.into_one().unwrap();
            Ok(one)
        };
    }

    async fn new_file(
        &self,
        parent_id: u64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta> {
        let result = self.file_meta_manager.new_file(
            parent_id, name, file_type.clone())
            .await;
        if let Err(e) = result {
            return Err(e);
        }
        let option = self.
            file_meta_manager
            .info_by_parent_and_name(parent_id, name)
            .await;
        if let None = option {
            return Err(ErrorInfo::new(12, "创文件失败"));
        }
        let f = option.unwrap();
        return Ok(f);
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
    async fn delete_file_blocks(&self, file_id: u64, block_index: i64) -> () {
        let result = self
            .file_block_meta_manager
            .delete_file_blocks(file_id, block_index)
            .await;
        match result {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    async fn info_by_id(&self, id: u64) -> ResponseResult<FileMeta> {
        return self.file_meta_manager.info_by_id(id).await;
    }

    async fn delete_file_meta(&self, id: u64) -> ResponseResult<FileMeta> {
        self
            .file_meta_manager
            .delete_file_meta(id)
            .await
    }
    async fn clean_file_meta(&self, id: u64) -> ResponseResult<Option<FileMeta>> {
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
    async fn file_block_meta(&self, file_meta_id: u64) -> Vec<FileBlockMeta> {
        return self
            .file_block_meta_manager
            .file_block_meta(file_meta_id)
            .await;
    }

    async fn file_block_meta_index(
        &self,
        file_meta_id: u64,
        block_index: i64,
    ) -> Option<FileBlockMeta> {
        return self
            .file_block_meta_manager
            .file_block_meta_index(file_meta_id, block_index)
            .await;
    }

    async fn file_block_meta_info_by_id(&self, id: i32) -> ResponseResult<Option<FileBlockMeta>> {
        return self
            .file_block_meta_manager
            .file_block_meta_info_by_id(id)
            .await;
    }

    async fn update_file_block_meta(
        &self,
        mut meta: FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        let option = self
            .file_block_meta_manager
            .update_file_block_meta(&mut meta)
            .await;
        return option;
    }

    async fn save_file_block_meta(&self, meta: FileBlockMeta) -> ResponseResult<Option<FileBlockMeta>> {
        let option = self
            .file_block_meta_manager
            .save_file_block_meta(meta)
            .await;
        return option;
    }

    async fn modified_blocks(&self, before: i64) -> ResponseResult<Vec<FileBlockMeta>> {
        let vec = self
            .file_block_meta_manager
            .modified_blocks(before)
            .await;
        return vec;
    }
}
