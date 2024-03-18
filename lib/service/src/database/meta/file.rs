use log::error;
use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::FileMeta;
use crate::CONTEXT;

use crate::database::meta::{FileBlockMeta, FileManager, FileMetaType};

pub(crate) mod file_block_meta;
pub(crate) mod file_meta;

pub struct SimpleFileManager {}

impl SimpleFileManager {
    pub(crate) fn new() -> Self {
        SimpleFileManager {}
    }
}

impl FileManager for SimpleFileManager {
    async fn list_deleted_file(&self, update_time: i64) -> Vec<FileMeta> {
        return CONTEXT
            .file_meta_manager
            .list_deleted_file(update_time)
            .await;
    }

    async fn list_by_parent(&self, parent_id: i32) -> ResponseResult<Vec<FileMeta>> {
        return CONTEXT.file_meta_manager.list_by_parent(parent_id).await;
    }
    async fn info_by_parent_and_name(&self, parent_id: i32, name: &str) -> Option<FileMeta> {
        return CONTEXT
            .file_meta_manager
            .info_by_parent_and_name(parent_id, name)
            .await;
    }

    async fn new_file(
        &self,
        parent_id: i32,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta> {
        let result = CONTEXT
            .file_meta_manager
            .new_file(parent_id, name, file_type.clone())
            .await;
        if let Err(e) = result {
            return Err(e);
        }
        let option = CONTEXT
            .file_meta_manager
            .info_by_parent_and_name(parent_id, name)
            .await;
        if let None = option {
            return Err(ErrorInfo::new(12, "创文件失败"));
        }
        let f = option.unwrap();
        // if file_type == FILE {
            // CONTEXT
            //     .file_block_meta_manager
            //     .new_file_block_meta(f.id.unwrap(), 0)
            //     .await
            //     .unwrap();
        // }
        return Ok(f);
    }

    async fn update_meta(&self, meta: FileMeta) -> Option<FileMeta> {
        let result = CONTEXT.file_meta_manager.update_meta(&meta).await;
        return match result {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                None
            }
        };
    }

    async fn update_file_content(&self, meta: FileMeta, block_index: usize) -> Option<FileMeta> {
        let result = CONTEXT.file_meta_manager.update_meta(&meta).await;
        let option = match result {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                return None;
            }
        };
        let result = CONTEXT
            .file_block_meta_manager
            .delete_file_blocks(meta.id.unwrap(), block_index as i64)
            .await;
        match result {
            Ok(_v) => {}
            Err(e) => {
                error!("{}", e);
            }
        };
        option
    }
    async fn delete_file_blocks(&self, file_id: i32, block_index: i64)->() {
        let result = CONTEXT
            .file_block_meta_manager
            .delete_file_blocks(file_id, block_index)
            .await;
        match result {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    async fn info_by_id(&self, id: i32) -> ResponseResult<Option<FileMeta>> {
        return CONTEXT.file_meta_manager.info_by_id(id).await;
    }

    async fn delete_file_meta(&self, id: i32) -> Option<FileMeta> {
        CONTEXT
            .file_meta_manager
            .delete_file_meta(id)
            .await
            .unwrap()
    }
    async fn clean_file_meta(&self, id: i32) -> ResponseResult<Option<FileMeta>> {
        let option = CONTEXT.file_meta_manager.info_by_id(id).await?;
        if let None = option {
            return Ok(None);
        }
        let file_meta = option.unwrap();
        CONTEXT
            .file_block_meta_manager
            .delete_file_meta_block_by_file_meta_id(id)
            .await
            ?;
        CONTEXT
            .file_meta_manager
            .delete_meta(&file_meta)
            .await
            ?;
        Ok(Some(file_meta))
    }
    async fn file_block_meta(&self, file_meta_id: i32) -> Vec<FileBlockMeta> {
        return CONTEXT
            .file_block_meta_manager
            .file_block_meta(file_meta_id)
            .await;
    }

    async fn file_block_meta_index(
        &self,
        file_meta_id: i32,
        block_index: i64,
    ) -> Option<FileBlockMeta> {
        return CONTEXT
            .file_block_meta_manager
            .file_block_meta_index(file_meta_id, block_index)
            .await;
    }

    async fn file_block_meta_info_by_id(&self, id: i32) -> Option<FileBlockMeta> {
        return CONTEXT
            .file_block_meta_manager
            .file_block_meta_info_by_id(id)
            .await;
    }

    async fn update_file_block_meta(
        &self,
        meta: FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>> {
        let option = CONTEXT
            .file_block_meta_manager
            .update_file_block_meta(meta)
            .await;
        return Ok(option);
    }

    async fn save_file_block_meta(&self, meta: FileBlockMeta) -> Option<FileBlockMeta> {
        let option = CONTEXT
            .file_block_meta_manager
            .save_file_block_meta(meta)
            .await;
        return option;
    }

    async fn modified_blocks(&self, _before: i64) -> Vec<FileBlockMeta> {
        let vec = CONTEXT
            .file_block_meta_manager
            .modified_blocks(_before)
            .await;
        return vec;
    }
}
