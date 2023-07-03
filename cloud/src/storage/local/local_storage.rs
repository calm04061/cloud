use crate::domain::table::tables::{CloudMeta, FileBlockMeta};
use crate::storage::storage::{
    CreateResponse, Quota, ResponseResult,
    StorageFile,
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
// #[async_trait::async_trait]
// impl Storage for LocalStorage {
//
//     async fn user_info(&mut self, _cloud_meta: CloudMeta) -> ResponseResult<User> {
//         todo!()
//     }
// }

impl Clone for LocalStorage {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[async_trait::async_trait]
impl StorageFile for LocalStorage {

    async fn upload_content(
        &mut self,
        file_block: FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let data_root = cloud_meta.data_root.unwrap();
        let path_str = format!("{}/{}", data_root, file_block.file_part_id);
        fs::write(path_str, content).await.ok();
        Ok(CreateResponse {
            domain_id: "".to_string(),
            drive_id: "".to_string(),
            encrypt_mode: "".to_string(),
            file_id: file_block.file_part_id.clone(),
            file_name: file_block.file_part_id.clone(),
            location: "".to_string(),
            parent_file_id: "".to_string(),
            rapid_upload: false,
            file_type: "".to_string(),
            upload_id: "".to_string(),
        })
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

    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        Ok(Quota {
            total: 1024 * 1024 * 1024,
            used: 0,
            remaining: 1024 * 1024 * 1024,
        })
    }
}
