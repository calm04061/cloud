use api::error::ErrorInfo;
use async_recursion::async_recursion;
use rbatis::RBatis;

use crate::meta::FileMetaType;
use crate::meta::FileMetaType::DIR;
use api::util::IntoOne;
use api::ResponseResult;
use persistence::meta::FileMeta;
use persistence::FileStatus;

#[derive(Clone)]
pub struct SimpleFileMetaManager {
    batis: RBatis,
}

impl SimpleFileMetaManager {
    pub fn new(batis: RBatis) -> Self {
        SimpleFileMetaManager { batis }
    }

    pub(crate) async fn delete_meta(&self, meta: &FileMeta) -> ResponseResult<u64> {
        Ok(FileMeta::delete_by_column(&self.batis.clone(), "id", meta.id)
            .await
            ?
            .rows_affected)
    }

    pub async fn update_meta(&self, meta: &FileMeta) -> ResponseResult<FileMeta> {
        FileMeta::update_by_column(&self.batis.clone(), meta, "id")
            .await
            ?;
        self.info_by_id(meta.id.unwrap()).await
    }

    pub(crate) async fn info_by_id(&self, id: u64) -> ResponseResult<FileMeta> {
        let vec = FileMeta::select_by_column(&self.batis.clone(), "id", id).await?;
        if vec.is_empty() {
            Err(ErrorInfo::new(7000, &format!("文件{id}")))
        } else {
            Ok(vec.into_one().unwrap())
        }
    }

    pub(crate) async fn list_by_parent(&self, parent_id: u64) -> ResponseResult<Vec<FileMeta>> {
        Ok(FileMeta::select_by_parent(&self.batis.clone(), parent_id)
            .await?)
    }

    pub(crate) async fn info_by_parent_and_name(
        &self,
        parent_id: u64,
        name: &str,
    ) -> ResponseResult<FileMeta> {
        let vec = FileMeta::info_by_parent_and_name(&self.batis.clone(), parent_id, name)
            .await
            ?;
        if vec.is_empty() {
            Err(ErrorInfo::new(7000, &format!("文件{name}")))
        } else {
            Ok(vec.into_one().unwrap())
        }
        // return if vec.is_empty() { None } else { vec.into_one() };
    }
    #[async_recursion]
    pub(crate) async fn delete_file_meta(&self, id: u64) -> ResponseResult<FileMeta> {
        // info!("delete file:{}", id);
        let file_meta = self.info_by_id(id).await?;

        let file_meta_type = file_meta.clone().file_type;
        if file_meta_type == DIR.get_code() {
            let vec = self.list_by_parent(file_meta.id.unwrap()).await?;
            for file_me in vec {
                let _result1 = self.delete_file_meta(file_me.id.unwrap()).await?;
            }
        }
        let mut file_meta = file_meta.clone();
        file_meta.deleted = 1;
        file_meta.update_time = chrono::Local::now().timestamp_millis();
        FileMeta::update_by_column(&self.batis.clone(), &file_meta, "id")
            .await?;

        Ok(file_meta)
    }

    pub(crate) async fn delete_one_file_meta(&self, id: u64) -> ResponseResult<FileMeta> {
        // info!("delete file:{}", id);
        let file_meta = self.info_by_id(id).await?;
        let mut file_meta = file_meta.clone();
        file_meta.deleted = 1;
        file_meta.update_time = chrono::Local::now().timestamp_millis();
        FileMeta::update_by_column(&self.batis.clone(), &file_meta, "id")
            .await?;

        Ok(file_meta)
    }
    pub(crate) async fn new_file(
        &self,
        parent_id: u64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<u64> {
        let value = FileMeta {
            id: None,
            name: name.to_string(),
            parent_id,
            file_type: file_type.get_code(),
            mode: 0o644,
            gid: 1000,
            uid: 1000,
            file_length: 0,
            status: FileStatus::Init.into(),
            deleted: 0,
            create_time: chrono::Local::now().timestamp_millis(),
            update_time: chrono::Local::now().timestamp_millis(),
        };
        Ok(FileMeta::insert(&self.batis.clone(), &value)
            .await
            ?
            .rows_affected)
    }
}
