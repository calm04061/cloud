use std::sync::OnceLock;

pub(crate) mod ali;
pub(crate) mod baidu;
pub(crate) mod local;
pub mod storage;
pub mod storage_facade;
pub(crate) mod onedrive;
#[cfg(not(windows))]
pub(crate) mod sftp;
pub mod web;
pub mod model;
pub mod repository;
mod google;

pub use repository::{CloudFileBlockRepository, CloudMetaRepository, ConfigRepository};
pub use storage_facade::StorageFacade;

static STORAGE_FACADE: OnceLock<StorageFacade> = OnceLock::new();

/// 宿主进程启动时注入依赖并初始化全局 facade。
/// 返回 false 表示重复初始化。
pub fn init_storage_facade(
    cloud_meta_repository: std::sync::Arc<dyn CloudMetaRepository>,
    cloud_file_block_repository: std::sync::Arc<dyn CloudFileBlockRepository>,
    config_repository: std::sync::Arc<dyn ConfigRepository>,
) -> bool {
    STORAGE_FACADE
        .set(StorageFacade::new(
            cloud_meta_repository,
            cloud_file_block_repository,
            config_repository,
        ))
        .is_ok()
}

/// 全局 StorageFacade；未初始化时 panic。
pub fn storage_facade() -> &'static StorageFacade {
    STORAGE_FACADE
        .get()
        .expect("STORAGE_FACADE 未初始化：请在 main 中调用 init_storage_facade")
}
