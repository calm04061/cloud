use std::future::Future;
use api::ResponseResult;
use persistence::{CloudMeta, FileBlockMeta, FileMeta, FileMetaType};

pub mod cloud;
pub mod file;

pub trait CloudMetaManager {
    fn add(&self, meta: &CloudMeta) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
    fn info(&self, id: i32) -> impl Future<Output=Option<CloudMeta>> + Send;

    fn update_meta(&self, meta: &CloudMeta) -> impl Future<Output=Option<CloudMeta>> + Send;
    fn delete(&self, id: i32) -> impl Future<Output=Option<CloudMeta>> + Send;
}

pub trait FileManager {
    fn list_deleted_file(&self, update_time: i64) -> impl Future<Output=Vec<FileMeta>> + Send;

    fn list_by_parent(&self, parent_id: i32) -> impl Future<Output=ResponseResult<Vec<FileMeta>>> + Send;

    fn info_by_parent_and_name(&self, parent_id: i32, name: &str) -> impl Future<Output=Option<FileMeta>> + Send;

    fn new_file(
        &self,
        parent_id: i32,
        name: &str,
        file_type: FileMetaType,
    ) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn update_meta(&self, meta: FileMeta) -> impl Future<Output=Option<FileMeta>> + Send;
    fn update_file_content(&self, meta: FileMeta, block_index: usize) -> impl Future<Output=Option<FileMeta>> + Send;
    fn delete_file_blocks(&self, id: i32, block_index: i64) -> impl Future<Output=()>;

    fn info_by_id(&self, id: i32) -> impl std::future::Future<Output=ResponseResult<Option<FileMeta>>> + Send;

    fn delete_file_meta(&self, id: i32) -> impl std::future::Future<Output=Option<FileMeta>> + Send;

    fn clean_file_meta(&self, id: i32) -> impl std::future::Future<Output=ResponseResult<Option<FileMeta>>> + Send;

    fn file_block_meta(&self, file_meta_id: i32) -> impl std::future::Future<Output=Vec<FileBlockMeta>> + Send;

    fn file_block_meta_index(&self, file_meta_id: i32, start: i64) -> impl std::future::Future<Output=Option<FileBlockMeta>> + Send;

    fn file_block_meta_info_by_id(&self, id: i32) -> impl std::future::Future<Output=Option<FileBlockMeta>> + Send;

    fn update_file_block_meta(&self, meta: FileBlockMeta) -> impl std::future::Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn save_file_block_meta(&self, meta: FileBlockMeta) -> impl std::future::Future<Output=Option<FileBlockMeta>> + Send;
    fn modified_blocks(&self, before: i64) -> impl std::future::Future<Output=Vec<FileBlockMeta>> + Send;
}
