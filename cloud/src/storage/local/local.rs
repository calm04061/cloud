use bytes::Bytes;
use crate::domain::table::tables::CloudMeta;
use crate::storage::storage::{StorageFile, CreateResponse, FileInfo, FileItemWrapper, Quota, ResponseResult, SearchResponse, Storage, User};

pub struct LocalStorage {}
#[async_trait::async_trait]
impl Storage for LocalStorage {
    fn phone_login(self) {
        todo!()
    }

    fn phone_code(self) {
        todo!()
    }

    fn password_login(self) {
        todo!()
    }

    async fn user_info(&mut self, _cloud_meta: CloudMeta) -> ResponseResult<User> {
        todo!()
    }
}
#[async_trait::async_trait]
impl StorageFile for LocalStorage {
    async fn upload(&mut self, _parent_file_id: &str, _name: &str, _file_path: &str, _cloud_meta: CloudMeta) -> ResponseResult<CreateResponse> {
        todo!()
    }

    async fn upload_content(&mut self, _parent_file_id: &str, _name: &str, _content: &Vec<u8>, _cloud_meta: CloudMeta) -> ResponseResult<CreateResponse> {
        todo!()
    }

    async fn search(&mut self, _parent_file_id: &str, _name: &str, _cloud_meta: CloudMeta) -> ResponseResult<SearchResponse> {
        todo!()
    }

    async fn delete(&mut self, _file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<()> {
        todo!()
    }

    async fn content(&mut self, _file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        todo!()
    }

    async fn info(&mut self, _file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        todo!()
    }

    async fn list(&mut self, _parent_file_id: &str, _cloud_meta: CloudMeta) -> ResponseResult<FileItemWrapper> {
        todo!()
    }

    async fn refresh_token(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<String> {
        todo!()
    }

    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        todo!()
    }

    fn authorize(&self, _server: &str, _id: i32) -> ResponseResult<String> {
        todo!()
    }

    async fn callback(&self, _server: String, _code: String, _id: i32) -> ResponseResult<String> {
        todo!()
    }
}