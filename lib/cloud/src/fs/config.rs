use log::{info, warn};
use service::CONTEXT;

use crate::fs::vfs::DEFAULT_TEMP_PATH;

pub(crate) const TEMP_PATH: &str = "TEMP_PATH";
pub(crate) const MOUNT_PATH: &str = "MOUNT_PATH";
pub(crate) const MAX_NUMBER_OF_SHARD: &str = "MAX_NUMBER_OF_SHARD";

/// 临时/缓存目录的统一读取入口。
///
/// 优先级：`.env` 的 `TEMP_PATH` > 数据库 `config` 表的 `TEMP_PATH` > 内置默认值。
/// 所有写入本地块缓存的路径（FUSE/DAV/NFS/scan/清理任务）必须经由本函数，
/// 避免多处配置不一致导致清理任务与实际缓存目录脱节。
pub async fn temp_path() -> String {
    if let Ok(value) = dotenvy::var(TEMP_PATH) {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return value;
        }
    }
    if let Some(config) = CONTEXT.config_manager.info(TEMP_PATH).await {
        let value = config.value.trim().to_string();
        if !value.is_empty() {
            return value;
        }
    }
    warn!(
        "TEMP_PATH 未配置，使用内置默认值 {}（建议在 .env 或数据库 config 表中显式配置）",
        DEFAULT_TEMP_PATH
    );
    DEFAULT_TEMP_PATH.to_string()
}

/// 启动时输出最终生效的关键配置摘要，并校验必填/可创建项。
pub async fn log_effective_config() {
    let resolved_temp_path = temp_path().await;
    let mount_path = CONTEXT
        .config_manager
        .info(MOUNT_PATH)
        .await
        .map(|c| c.value);
    let cloud_file_root = CONTEXT
        .config_manager
        .info(api::CLOUD_FILE_ROOT)
        .await
        .map(|c| c.value);
    let shard = dotenvy::var(MAX_NUMBER_OF_SHARD).unwrap_or_else(|_| "3".to_string());

    if mount_path.is_none() {
        warn!("MOUNT_PATH 未配置：FUSE 挂载功能将不可用（/api/fs/mount 会失败）");
    }
    if cloud_file_root.is_none() {
        warn!("CLOUD_FILE_ROOT 未配置：云存储根路径将无法解析");
    }

    if let Err(e) = std::fs::create_dir_all(resolved_temp_path.as_str()) {
        panic!(
            "TEMP_PATH {} 创建失败：{}（请检查 .env/数据库 config 中的 TEMP_PATH）",
            resolved_temp_path, e
        );
    }

    info!(
        "effective config: http_port={}, nfs_bind_addr={}, public_base_url={}, temp_path={}, mount_path={:?}, cloud_file_root={:?}, max_number_of_shard={}",
        dotenvy::var("HTTP_PORT").unwrap_or_else(|_| "8088".to_string()),
        dotenvy::var("NFS_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:11111".to_string()),
        dotenvy::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "(未配置，使用请求 Host)".to_string()),
        resolved_temp_path,
        mount_path,
        cloud_file_root,
        shard,
    );
}
