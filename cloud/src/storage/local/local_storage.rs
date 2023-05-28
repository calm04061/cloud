use crate::domain::table::tables::CloudMeta;
use crate::storage::storage::{
    CreateResponse, FileInfo, FileItemWrapper, Quota, ResponseResult, SearchResponse, Storage,
    StorageFile, User,
};
use bytes::Bytes;
use log::info;
use std::path::Path;
use tokio::fs;

pub struct LocalStorage {}
impl LocalStorage {
    pub(crate) fn new() -> LocalStorage {
        LocalStorage {}
    }
}
#[async_trait::async_trait]
impl Storage for LocalStorage {

    async fn user_info(&mut self, _cloud_meta: CloudMeta) -> ResponseResult<User> {
        todo!()
    }
}

impl Clone for LocalStorage {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
impl StorageFile for LocalStorage {

    async fn upload_content(
        &mut self,
        file_id: &str,
        content: &Vec<u8>,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let data_root = cloud_meta.data_root.unwrap();
        let path_str = format!("{}/{}", data_root, file_id);
        fs::write(path_str, content).await.ok();
        Ok(CreateResponse {
            domain_id: "".to_string(),
            drive_id: "".to_string(),
            encrypt_mode: "".to_string(),
            file_id: file_id.to_string(),
            file_name: file_id.to_string(),
            location: "".to_string(),
            parent_file_id: "".to_string(),
            rapid_upload: false,
            file_type: "".to_string(),
            upload_id: "".to_string(),
        })
    }

    async fn search(
        &mut self,
        _parent_file_id: &str,
        _name: &str,
        _cloud_meta: CloudMeta,
    ) -> ResponseResult<SearchResponse> {
        todo!()
    }

    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()> {
        let data_root = cloud_meta.data_root.unwrap();
        let path_str = format!("{}/{}", data_root, file_id);
        let path = Path::new(path_str.as_str());
        if path.exists() {
            fs::remove_file(path).await.ok();
        } else {
            info!("文件不存在");
        }
        Ok(())
    }

    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        let data_root = cloud_meta.data_root.unwrap();
        let path_str = format!("{}/{}", data_root, file_id);
        let result = fs::read(path_str).await;
        let vec = result.unwrap();
        let bytes = Bytes::from(vec);
        Ok(bytes)
    }

    async fn info(&mut self, _file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        todo!()
    }

    async fn list(
        &mut self,
        _parent_file_id: &str,
        _cloud_meta: CloudMeta,
    ) -> ResponseResult<FileItemWrapper> {
        todo!()
    }

    async fn refresh_token(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<String> {
        Ok("{}".to_string())
    }

    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        Ok(Quota {
            total: 1024 * 1024 * 1024,
            used: 0,
            remaining: 1024 * 1024 * 1024,
        })
    }

    fn authorize(&self, _server: &str, _id: i32) -> ResponseResult<String> {
        todo!()
    }

    async fn callback(&self, _server: String, _code: String, _id: i32) -> ResponseResult<String> {
        todo!()
    }

    async fn after_callback(&mut self, _cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        todo!()
    }

    fn client_id(&self) -> String {
        todo!()
    }

    fn client_secret(&self) -> String {
        todo!()
    }
}
