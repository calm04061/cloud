use api::ResponseResult;
use persistence::meta::CloudMeta;
use persistence::CloudFileBlock;

/// 云账号元数据的持久化接口。
///
/// storage 层不依赖任何具体的数据库实现（service/CONTEXT），
/// 由宿主进程（lib/cloud）在启动时注入，见 `init_storage_facade`。
#[async_trait::async_trait]
pub trait CloudMetaRepository: Send + Sync {
    async fn info(&self, id: i32) -> ResponseResult<CloudMeta>;
    async fn update_meta(&self, cloud_meta: &CloudMeta) -> ResponseResult<CloudMeta>;
    async fn select_by_status(&self, status: i8) -> ResponseResult<Vec<CloudMeta>>;
}

/// 云端块副本记录读取接口。
#[async_trait::async_trait]
pub trait CloudFileBlockRepository: Send + Sync {
    async fn select_by_file_block_id(&self, file_block_id: i32) -> ResponseResult<Vec<CloudFileBlock>>;
}

/// 配置表读取接口。
#[async_trait::async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn info(&self, property: &str) -> Option<String>;
}
