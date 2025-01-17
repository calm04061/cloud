use crate::storage::storage::ResponseResult;
use persistence::{CloudMeta, FileBlockMeta, FileMeta, FileMetaType};

pub mod cloud;
pub mod file;

#[async_trait::async_trait]
pub trait CloudMetaManager {
    async fn add(&self, meta: &CloudMeta) -> ResponseResult<CloudMeta>;
    async fn info(&self, id: i32) -> Option<CloudMeta>;
    async fn update_meta(&self, meta: &CloudMeta) -> Option<CloudMeta>;
    async fn delete(&self, id: i32) -> Option<CloudMeta>;
}

#[async_trait::async_trait]
pub trait FileManager {
    async fn list_deleted_file(&self, update_time: i64) -> Vec<FileMeta>;

    async fn list_by_parent(&self, parent_id: i32) -> ResponseResult<Vec<FileMeta>>;

    async fn info_by_parent_and_name(&self, parent_id: i32, name: &str) -> Option<FileMeta>;

    async fn new_file(
        &self,
        parent_id: i32,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta>;

    async fn update_meta(&self, meta: FileMeta) -> Option<FileMeta>;
    async fn update_file_content(&self, meta: FileMeta, block_index: usize) -> Option<FileMeta>;
    async fn delete_file_blocks(&self, id: i32, block_index: i64);

    async fn info_by_id(&self, id: i32) -> ResponseResult<Option<FileMeta>>;

    async fn delete_file_meta(&self, id: i32) -> Option<FileMeta>;

    async fn clean_file_meta(&self, id: i32) -> ResponseResult<Option<FileMeta>>;

    async fn file_block_meta(&self, file_meta_id: i32) -> Vec<FileBlockMeta>;

    async fn file_block_meta_index(&self, file_meta_id: i32, start: i64) -> Option<FileBlockMeta>;

    async fn file_block_meta_info_by_id(&self, id: i32) -> Option<FileBlockMeta>;

    async fn update_file_block_meta(&self, meta: FileBlockMeta) -> ResponseResult<Option<FileBlockMeta>>;

    async fn save_file_block_meta(&self, meta: FileBlockMeta) -> Option<FileBlockMeta>;
    async fn modified_blocks(&self, before: i64) -> Vec<FileBlockMeta>;
}
