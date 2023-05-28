use crate::domain::table::tables::{CloudMeta, FileBlockMeta, FileMeta};
use crate::storage::storage::ResponseResult;

pub mod cloud;
pub mod file;

pub(crate) enum FileStatus {
    Init,
    Uploading,
    UploadSuccess,
    UploadFail,
    FileNotExist,
    FileReadError,
    WaitClean,
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum FileMetaType {
    FILE,
    DIR,
    SYMLINK,
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy)]
pub enum CloudType {
    AliYun,
    Baidu,
    Local,
    OneDrive,
}

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

    async fn list_by_parent(&self, parent_id: i64) -> ResponseResult<Vec<FileMeta>>;

    async fn info_by_parent_and_name(&self, parent_id: i64, name: &str) -> Option<FileMeta>;

    async fn new_file(
        &self,
        parent_id: i64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta>;

    async fn update_meta(&self, meta: FileMeta) -> Option<FileMeta>;
    async fn update_file_content(&self, meta: FileMeta, block_index: usize) -> Option<FileMeta>;
    async fn delete_file_blocks(&self, id: i64, block_index: i64);

    async fn info_by_id(&self, id: i64) -> ResponseResult<Option<FileMeta>>;

    async fn delete_file_meta(&self, id: i64) -> Option<FileMeta>;

    async fn clean_file_meta(&self, id: i64) -> ResponseResult<Option<FileMeta>>;

    async fn file_block_meta(&self, file_meta_id: i64) -> Vec<FileBlockMeta>;

    async fn file_block_meta_index(&self, file_meta_id: i64, start: i64) -> Option<FileBlockMeta>;

    async fn file_block_meta_info_by_id(&self, id: i64) -> Option<FileBlockMeta>;

    async fn update_file_block_meta(
        &self,
        meta: FileBlockMeta,
    ) -> ResponseResult<Option<FileBlockMeta>>;

    // fn file_blocks(&self, file_block_meta_id: i64) -> Vec<FileBlock>;

    async fn new_file_block_meta(
        &self,
        file_meta_id: i64,
        block_index: i64,
    ) -> Option<FileBlockMeta>;

    async fn modified_blocks(&self, before: i64) -> Vec<FileBlockMeta>;
}
