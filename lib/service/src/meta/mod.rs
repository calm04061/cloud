use api::ResponseResult;
use chrono::{DateTime, Utc};
use persistence::meta::{CloudMeta, FileBlockMeta, FileMeta};
use persistence::FileMetaType;
use std::future::Future;

pub mod cloud;
pub mod file;

pub trait CloudMetaManager {
    fn add(&self, meta: &CloudMeta) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
    fn info(&self, id: i32) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;

    fn update_meta(&self, meta: &CloudMeta) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
    fn delete(&self, id: i32) -> impl Future<Output=ResponseResult<CloudMeta>> + Send;
}

pub trait FileManager {
    /// 捞取待清理的软删除文件。
    ///
    /// - update_time：宽限期截止（早于该时间的才处理，避免 VFS 删除尚未落库完成）
    /// - pre_id：游标分页
    /// - stale_time：抢占中的文件（status=Cleaning）超过该时间的视为死锁，允许重新抢占
    fn list_deleted_file(&self, update_time: DateTime<Utc>, pre_id: i64, stale_time: DateTime<Utc>) -> impl Future<Output=ResponseResult<Vec<FileMeta>>> + Send;

    fn list_by_parent(&self, parent_id: i64) -> impl Future<Output=ResponseResult<Vec<FileMeta>>> + Send;

    fn list_by_parent_page(&self, parent_id: i64, start: i64, size: usize) -> impl Future<Output=ResponseResult<(Vec<FileMeta>, bool)>> + Send;

    fn info_by_parent_and_name(&self, parent_id: i64, name: &str) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn new_file(
        &self,
        parent_id: i64,
        name: &str,
        file_type: FileMetaType,
    ) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn update_meta(&self, meta: FileMeta) -> impl Future<Output=ResponseResult<FileMeta>> + Send;
    fn update_file_content(&self, meta: FileMeta, block_index: usize) -> impl Future<Output=ResponseResult<FileMeta>> + Send;
    fn delete_file_blocks(&self, id: i64, block_index: i64) -> impl Future<Output=ResponseResult<u64>>;

    fn info_by_id(&self, id: i64) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    // fn delete_file_meta(&self, id: u64) -> impl Future<Output=ResponseResult<FileMeta>> + Send;
    fn delete_one_file_meta(&self, id: i64) -> impl Future<Output=ResponseResult<FileMeta>> + Send;

    fn clean_file_meta(&self, id: i64) -> impl Future<Output=ResponseResult<Option<FileMeta>>> + Send;

    fn file_block_meta(&self, file_meta_id: i64) -> impl Future<Output=ResponseResult<Vec<FileBlockMeta>>> + Send;

    fn file_block_meta_index(&self, file_meta_id: i64, start: i64) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn file_block_meta_info_by_id(&self, id: i32) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn update_file_block_meta(&self, meta: FileBlockMeta) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;

    fn save_file_block_meta(&self, meta: FileBlockMeta) -> impl Future<Output=ResponseResult<Option<FileBlockMeta>>> + Send;
    fn modified_blocks(&self, before: i64) -> impl Future<Output=ResponseResult<Vec<FileBlockMeta>>> + Send;
}
