use std::future::Future;

use api::ResponseResult;
use persistence::{CloudMeta, FileBlockMeta, FileMeta, FileMetaType};

pub mod cloud;
pub mod file;

pub trait CloudMetaManager {
    fn add(&self, meta: &CloudMeta) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
    fn info(&self, id: i32) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;

    fn update_meta(&self, meta: &CloudMeta) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
    fn delete(&self, id: i32) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
}

pub trait FileManager {
    fn list_deleted_file(&self, update_time: i64) -> impl Future<Output=ResponseResult<Vec<FileMeta>>> + Send;

    fn list_by_parent(&self, parent_id: u64) -> impl Future<Output=ResponseResult<Vec<FileMeta>>> + Send;

    fn list_by_parent_page(&self, parent_id: u64, start: u64, size: usize) -> impl Future<Output=ResponseResult<(Vec<FileMeta>, bool)>> + Send;

    fn info_by_parent_and_name(&self, parent_id: u64, name: &str) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn new_file(
        &self,
        parent_id: u64,
        name: &str,
        file_type: FileMetaType,
    ) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn update_meta(&self, meta: FileMeta) -> impl Future<Output=ResponseResult<FileMeta>> + Send;
    fn update_file_content(&self, meta: FileMeta, block_index: usize) -> impl Future<Output=ResponseResult<FileMeta>> + Send;
    fn delete_file_blocks(&self, id: u64, block_index: i64) -> impl Future<Output=()>;

    fn info_by_id(&self, id: u64) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn delete_file_meta(&self, id: u64) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn clean_file_meta(&self, id: u64) -> impl Future<Output=ResponseResult<Option<FileMeta>>> + Send;

    fn file_block_meta(&self, file_meta_id: u64) -> impl Future<Output=Vec<FileBlockMeta>> + Send;

    fn file_block_meta_index(&self, file_meta_id: u64, start: i64) -> impl Future<Output=Option<FileBlockMeta>> + Send;

    fn file_block_meta_info_by_id(&self, id: i32) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn update_file_block_meta(&self, meta: FileBlockMeta) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn save_file_block_meta(&self, meta: FileBlockMeta) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;
    fn modified_blocks(&self, before: i64) -> impl Future<Output=ResponseResult<Vec<FileBlockMeta>>> + Send;
}
