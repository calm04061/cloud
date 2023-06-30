use crypto::digest::Digest;
use crypto::md5::Md5;
use log::debug;
use rbatis::utils::into_one::IntoOne;
use rbs::to_value;

use crate::database::meta::cloud::MetaStatus;
use crate::database::meta::{FileBlockMeta, FileStatus};
use crate::domain::table::tables::{CloudFileBlock, CloudMeta};
use crate::pool;
use crate::storage::storage::ResponseResult;

pub struct SimpleFileBlockMetaManager {}

impl SimpleFileBlockMetaManager {
    pub(crate) fn new() -> Self {
        SimpleFileBlockMetaManager {}
    }
    pub(crate) async fn file_block_meta_info_by_id(&self, id: i32) -> Option<FileBlockMeta> {
        let rows = FileBlockMeta::select_by_column(pool!(), "id", id)
            .await
            .unwrap();

        if rows.is_empty() {
            None
        } else {
            rows.into_one()
        }
    }
    pub(crate) async fn update_file_block_meta(
        &self,
        mut meta: FileBlockMeta,
    ) -> Option<FileBlockMeta> {
        meta.update_time = chrono::Local::now().timestamp_millis();
        FileBlockMeta::update_by_column(pool!(), &meta, "id")
            .await
            .unwrap();
        return self.file_block_meta_info_by_id(meta.id.unwrap()).await;
    }

    pub(crate) async fn new_file_block_meta(
        &self,
        file_meta_id: i32,
        block_index: i64,
    ) -> Option<FileBlockMeta> {
        let option = self.file_block_meta_index(file_meta_id, block_index).await;
        if let Some(file_block) = option {
            return Some(file_block);
        }
        let mut md5 = Md5::new();
        let file_name = format!("{}:{}", file_meta_id, block_index);
        let file_name_body = file_name.as_bytes();
        md5.input(file_name_body);
        let file_name_hash = md5.result_str();
        let value = FileBlockMeta {
            id: None,
            file_part_id: file_name_hash,
            block_index,
            update_time: chrono::Local::now().timestamp_millis(),
            file_modify_time: chrono::Local::now().timestamp_millis(),
            file_meta_id,
            deleted: 0,
            part_hash: None,
            status: FileStatus::Init.into(),
        };
        FileBlockMeta::insert(pool!(), &value).await.unwrap();
        let option = self.file_block_meta_index(file_meta_id, block_index).await;
        let meta = option.unwrap();

        let file_block_meta_id = meta.id.unwrap();
        let vec = CloudMeta::quota_random(pool!(), MetaStatus::Enable.into(), 3)
            .await
            .unwrap();
        for cloud in vec {
            let block = CloudFileBlock::init(file_block_meta_id, cloud.id.unwrap());
            CloudFileBlock::insert(pool!(), &block).await.unwrap();
        }
        Some(meta)
    }

    pub(crate) async fn file_block_meta_index(
        &self,
        file_meta_id: i32,
        block_index: i64,
    ) -> Option<FileBlockMeta> {
        let vec = FileBlockMeta::select_by_file_meta_id_and_block_index(
            pool!(),
            file_meta_id,
            block_index,
        )
        .await
        .unwrap();
        if vec.is_empty() {
            return None;
        } else {
            vec.into_one()
        }
    }

    pub(crate) async fn file_block_meta(&self, file_meta_id: i32) -> Vec<FileBlockMeta> {
        FileBlockMeta::select_by_file_meta_id(pool!(), file_meta_id)
            .await
            .unwrap()
    }

    pub(crate) async fn modified_blocks(&self, _before: i64) -> Vec<FileBlockMeta> {
        return pool!().query_decode("select * from file_block_meta where (part_hash <> cloud_file_hash or cloud_file_hash is null ) and deleted = 0 order by update_time,id",vec![]).await.unwrap();
    }

    pub(crate) async fn delete_file_blocks(
        &self,
        file_id: i32,
        block_index: i64,
    ) -> ResponseResult<u64> {
        Ok(pool!().exec("update file_block_meta set deleted = 1,update_time=? where file_meta_id=? and block_index>? and deleted = 0",vec![to_value!(chrono::Local::now().timestamp_millis()),to_value!(file_id),to_value!(block_index)]).await.unwrap().rows_affected)
    }

    pub(crate) async fn delete_file_meta_block_by_file_meta_id(
        &self,
        file_meta_id: i32,
    ) -> ResponseResult<u64> {
        let vec = FileBlockMeta::select_by_column(pool!(), "file_meta_id", file_meta_id)
            .await
            .unwrap();
        for meta in vec {
            CloudFileBlock::delete_by_column(pool!(), "cloud_meta_id", meta.id.unwrap())
                .await
                .unwrap();
        }
        debug!("delete block meta by file id:{}", file_meta_id);
        Ok(
            FileBlockMeta::delete_by_column(pool!(), "file_meta_id", file_meta_id)
                .await
                .unwrap()
                .rows_affected,
        )
    }
}
