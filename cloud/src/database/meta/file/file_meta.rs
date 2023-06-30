use log::info;
use rbatis::utils::into_one::IntoOne;

use crate::database::meta::FileMetaType::DIR;
use crate::database::meta::{FileMetaType, FileStatus};
use crate::domain::table::tables::FileMeta;
use crate::pool;
use crate::storage::storage::ResponseResult;

pub struct SimpleFileMetaManager {}

impl SimpleFileMetaManager {
    pub(crate) fn new() -> Self {
        SimpleFileMetaManager {}
    }

    pub(crate) async fn delete_meta(&self, meta: &FileMeta) -> ResponseResult<u64> {
        Ok(FileMeta::delete_by_column(pool!(), "id", meta.id)
            .await
            .unwrap()
            .rows_affected)
    }
    pub(crate) async fn list_deleted_file(&self, update_time: i64) -> Vec<FileMeta> {
        // let connection = pool.get().unwrap();
        return FileMeta::list_deleted_file(pool!(), update_time)
            .await
            .unwrap();
    }
    pub(crate) async fn update_meta(&self, meta: &FileMeta) -> ResponseResult<Option<FileMeta>> {
        FileMeta::update_by_column(pool!(), meta, "id")
            .await
            .unwrap();
        return self.info_by_id(meta.id.unwrap()).await;
    }

    pub(crate) async fn info_by_id(&self, id: i32) -> ResponseResult<Option<FileMeta>> {
        let vec = FileMeta::select_by_column(pool!(), "id", id).await.unwrap();
        return if vec.is_empty() {
            Ok(None)
        } else {
            Ok(vec.into_one())
        };
    }

    pub(crate) async fn list_by_parent(&self, parent_id: i32) -> ResponseResult<Vec<FileMeta>> {
        Ok(FileMeta::select_by_parent(pool!(), parent_id)
            .await
            .unwrap())
    }

    pub(crate) async fn info_by_parent_and_name(
        &self,
        parent_id: i32,
        name: &str,
    ) -> Option<FileMeta> {
        let vec = FileMeta::info_by_parent_and_name(pool!(), parent_id, name)
            .await
            .unwrap();
        return if vec.is_empty() { None } else { vec.into_one() };
    }
    #[async_recursion::async_recursion]
    pub(crate) async fn delete_file_meta(&self, id: i32) -> ResponseResult<Option<FileMeta>> {
        info!("delete file:{}", id);
        let file_meta = self.info_by_id(id).await.unwrap();

        let result = if let Some(r) = file_meta {
            let file_meta_type = r.file_type;
            if file_meta_type == DIR.get_code() {
                let vec = self.list_by_parent(r.id.unwrap()).await.unwrap();
                for file_me in vec {
                    let _result1 = self.delete_file_meta(file_me.id.unwrap()).await.unwrap();
                }
            }
            Some(r)
        } else {
            None
        };
        let mut file_meta = result.clone().unwrap();
        file_meta.deleted = 1;
        file_meta.update_time = chrono::Local::now().timestamp_millis();
        FileMeta::update_by_column(pool!(), &file_meta, "id")
            .await
            .unwrap();

        return Ok(result);
    }
    pub(crate) async fn new_file(
        &self,
        parent_id: i32,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<u64> {
        let value = FileMeta {
            id: None,
            name: name.to_string(),
            parent_id,
            file_type: file_type.get_code(),
            file_length: 0,
            status: FileStatus::Init.into(),
            deleted: 0,
            create_time: chrono::Local::now().timestamp_millis(),
            update_time: chrono::Local::now().timestamp_millis(),
        };
        Ok(FileMeta::insert(pool!(), &value)
            .await
            .unwrap()
            .rows_affected)
    }
}
