use std::sync::Arc;

use api::ResponseResult;
use persistence::meta::CloudMeta;
use persistence::CloudFileBlock;
use service::meta::CloudMetaManager;
use service::CONTEXT;
use storage::{CloudFileBlockRepository, CloudMetaRepository, ConfigRepository};

/// 基于 service CONTEXT 的仓储实现，在 main 中注入给 storage 层。
pub struct ServiceCloudMetaRepository;

#[async_trait::async_trait]
impl CloudMetaRepository for ServiceCloudMetaRepository {
    async fn info(&self, id: i32) -> ResponseResult<CloudMeta> {
        CONTEXT.cloud_meta_manager.info(id).await
    }

    async fn update_meta(&self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta> {
        CONTEXT.cloud_meta_manager.update_meta(cloud_meta).await?;
        Ok(cloud_meta.clone())
    }

    async fn select_by_status(&self, status: i8) -> ResponseResult<Vec<CloudMeta>> {
        CONTEXT.cloud_meta_manager.select_by_status(status).await
    }
}

pub struct ServiceCloudFileBlockRepository;

#[async_trait::async_trait]
impl CloudFileBlockRepository for ServiceCloudFileBlockRepository {
    async fn select_by_file_block_id(&self, file_block_id: i32) -> ResponseResult<Vec<CloudFileBlock>> {
        CONTEXT
            .cloud_file_block_manager
            .select_by_file_block_id(file_block_id)
            .await
    }
}

pub struct ServiceConfigRepository;

#[async_trait::async_trait]
impl ConfigRepository for ServiceConfigRepository {
    async fn info(&self, property: &str) -> Option<String> {
        CONTEXT
            .config_manager
            .info(property)
            .await
            .map(|config| config.value)
    }
}

pub fn init_storage_repositories() -> bool {
    storage::init_storage_facade(
        Arc::new(ServiceCloudMetaRepository),
        Arc::new(ServiceCloudFileBlockRepository),
        Arc::new(ServiceConfigRepository),
    )
}
