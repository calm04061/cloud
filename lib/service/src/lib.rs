use crate::config::ConfigManager;
use api::ResponseResult;
use once_cell::sync::Lazy;
use persistence::ApplicationConfig;
use sqlx::Pool;
use sqlx_sqlite::{Sqlite, SqliteConnectOptions, SqlitePoolOptions};
use std::fs;
use std::str::FromStr;
use crate::cloud_file_block_manager::CloudFileBlockManager;
use crate::meta::cloud::SimpleCloudMetaManager;
use crate::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::meta::file::file_meta::SimpleFileMetaManager;
use crate::meta::file::SimpleFileManager;
use crate::migrate::{query_version, Version};
use crate::temp_cloud_file_manager::TempCloudFileManager;
use crate::user_manager::UserManager;
use event_message::EventMessageManager;
use crate::migrate::version::default_version_migrate::DefaultVersionMigrate;
use crate::migrate::versions_migrate::VersionsMigrate;

pub mod meta;
pub mod config;
pub mod user_manager;
pub mod event_message;
pub mod cloud_file_block_manager;
mod temp_cloud_file_manager;
mod migrate;

pub type DbPool = Pool<Sqlite>;
pub static CONTEXT: Lazy<ServiceContext> = Lazy::new(|| ServiceContext::default());

pub struct ServiceContext {
    pub config: ApplicationConfig,
    pub pool: Pool<Sqlite>,
    pub cloud_meta_manager: SimpleCloudMetaManager,
    pub file_meta_manager: SimpleFileMetaManager,
    pub file_block_meta_manager: SimpleFileBlockMetaManager,
    pub file_manager: SimpleFileManager,
    pub config_manager: ConfigManager,
    pub event_message_manager: EventMessageManager,
    pub cloud_file_block_manager: CloudFileBlockManager,
    pub user_manager: UserManager,
    pub temp_cloud_file_manager: TempCloudFileManager,
}

impl ServiceContext {
    pub fn init_pool() {
        if let Err(e) = fs::create_dir_all("./data") {
            log::error!("Failed to create data directory: {}", e);
        }
    }
    pub async fn upgrade(&self) {
        let result: ResponseResult<String> = query_version(self.pool.clone()).await;
        let pool = self.pool.clone();
        let mut transaction = match pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                log::error!("Failed to begin transaction: {}", e);
                return;
            }
        };

        let pre_version = match result {
            Ok(v) => v,
            Err(_e) => "0.0.0".to_string(),
        };
        let pre_version = match pre_version.parse::<Version>() {
            Ok(v) => v,
            Err(e) => {
                log::error!("Failed to parse version: {:?}", e);
                let _ = transaction.rollback().await;
                return;
            }
        };
        let mut versions_migrate = VersionsMigrate::new(pre_version);
        versions_migrate.add(Box::new(DefaultVersionMigrate::new("0.0.1", include_str!("migrate/version/0.0.1.sql")))).await;
        versions_migrate.add(Box::new(DefaultVersionMigrate::new("0.0.2", include_str!("migrate/version/0.0.2.sql")))).await;
        versions_migrate.add(Box::new(DefaultVersionMigrate::new("0.0.3", include_str!("migrate/version/0.0.3.sql")))).await;
        let migrate_result = versions_migrate.do_migrate(&mut transaction).await;

        match migrate_result {
            Ok(_) => {
                if let Err(e) = transaction.commit().await {
                    log::error!("Failed to commit transaction: {}", e);
                }
            }
            Err(e) => {
                log::error!("Migration failed: {}", e);
                if let Err(e) = transaction.rollback().await {
                    log::error!("Failed to rollback transaction: {}", e);
                }
            }
        }
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        Self::init_pool();
        let config = ApplicationConfig::default();
        let options = match SqliteConnectOptions::from_str(config.database_url.as_str()) {
            Ok(o) => o,
            Err(e) => {
                log::error!("Failed to parse database URL: {}", e);
                panic!("Failed to parse database URL: {}", e);
            }
        };
        let db_pool = SqlitePoolOptions::new().connect_lazy_with(options);

        let file_meta_manager = SimpleFileMetaManager::new(db_pool.clone());
        let cloud_file_block_manager = CloudFileBlockManager::new(db_pool.clone());
        let file_block_meta_manager = SimpleFileBlockMetaManager::new(db_pool.clone());

        ServiceContext {
            config,
            cloud_meta_manager: SimpleCloudMetaManager::new(db_pool.clone()),
            file_meta_manager: file_meta_manager.clone(),
            file_block_meta_manager: file_block_meta_manager.clone(),
            file_manager: SimpleFileManager::new(db_pool.clone(), file_meta_manager.clone(), file_block_meta_manager.clone()),
            config_manager: ConfigManager::new(db_pool.clone()),
            event_message_manager: EventMessageManager::new(db_pool.clone()),
            cloud_file_block_manager,
            user_manager: UserManager::new(db_pool.clone()),
            pool: db_pool.clone(),
            temp_cloud_file_manager: TempCloudFileManager::new(db_pool.clone()),
        }
    }
}
