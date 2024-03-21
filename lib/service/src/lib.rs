use std::fs;

use once_cell::sync::Lazy;
use rbatis::RBatis;
use rbatis::table_sync::{SqliteTableMapper, sync};
use api::ROOT_FILE_ID;

use persistence::{ApplicationConfig, CloudFileBlock, CloudMeta, Config, EventMessage, FileBlockMeta, FileMeta, FileMetaType, FileStatus, User};

use crate::database::cloud_file_block_manager::CloudFileBlockManager;
use crate::database::config::ConfigManager;
use crate::database::event_message::EventMessageManager;
use crate::database::meta::cloud::SimpleCloudMetaManager;
use crate::database::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::database::meta::file::file_meta::SimpleFileMetaManager;
use crate::database::meta::file::SimpleFileManager;
use crate::database::user_manager::UserManager;

pub mod database;

pub static CONTEXT: Lazy<ServiceContext> = Lazy::new(|| ServiceContext::default());

pub struct ServiceContext {
    pub config: ApplicationConfig,
    pub rb: RBatis,
    pub cloud_meta_manager: SimpleCloudMetaManager,
    pub file_meta_manager: SimpleFileMetaManager,
    pub file_block_meta_manager: SimpleFileBlockMetaManager,
    pub file_manager: SimpleFileManager,
    pub config_manager: ConfigManager,
    pub event_message_manager: EventMessageManager,
    pub cloud_file_block_manager: CloudFileBlockManager,
    pub user_manager: UserManager,
}

impl ServiceContext {
    /// init database pool
    pub async fn init_pool(&self) {
        fs::create_dir_all("./data").unwrap();
        log::info!("rbatis pool init ({})...", self.config.database_url);
        let driver = rbdc_sqlite::driver::SqliteDriver {};
        let driver_name = format!("{:?}", driver);
        self.rb
            .init(driver, &self.config.database_url)
            .expect("rbatis pool init fail!");
        self.rb.acquire().await.expect(&format!(
            "rbatis connect database(driver={},url={}) fail",
            driver_name, self.config.database_url
        ));
        log::info!(
            "rbatis pool init success! pool state = {:?}",
            self.rb.get_pool().expect("pool not init!").state().await
        );
    }
    pub async fn upgrade(&self) {
        // s.sql_id = " PRIMARY KEY AUTOINCREMENT NOT NULL ".to_string();
        let sqlite_table_mapper = SqliteTableMapper {};
        let conn = self.rb.acquire().await.unwrap();
        sync(&conn, &sqlite_table_mapper, Config::sync_default(), "config").await.unwrap();
        sync(&conn, &sqlite_table_mapper, CloudMeta::sync_default(), "cloud_meta").await.unwrap();
        sync(&conn, &sqlite_table_mapper, FileMeta::sync_default(), "file_meta").await.unwrap();
        sync(&conn, &sqlite_table_mapper, CloudFileBlock::sync_default(), "cloud_file_block").await.unwrap();
        sync(&conn, &sqlite_table_mapper, FileBlockMeta::sync_default(), "file_block_meta").await.unwrap();
        sync(&conn, &sqlite_table_mapper, EventMessage::sync_default(), "event_message").await.unwrap();
        sync(&conn, &sqlite_table_mapper, User::sync_default(), "user").await.unwrap();

        let vec = FileMeta::select_by_column(&conn, "id", 1).await.unwrap();
        if vec.is_empty() {
            let file_meta = FileMeta {
                id: Some(ROOT_FILE_ID),
                name: "/".to_string(),
                parent_id: 0,
                file_type: FileMetaType::DIR.get_code(),
                mode: 0o755,
                gid: 1000,
                uid: 1000,
                file_length: 0,
                status: FileStatus::UploadSuccess.into(),
                deleted: 0,
                create_time: 0,
                update_time: 0,
            };
            FileMeta::insert(&conn, &file_meta).await.unwrap();
        }
        let vec = User::select_by_column(&conn, "username", "admin").await.unwrap();
        if vec.is_empty() {
            let user = User{
                id: None,
                username: "admin".to_string(),
                password: "admin".to_string(),
            };
            User::insert(&conn,&user).await.unwrap();
        }
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        let config = ApplicationConfig::default();
        let batis = persistence::support::application_config::init_rbatis(&config);
        let file_meta_manager = SimpleFileMetaManager::new(batis.clone());
        let file_block_meta_manager = SimpleFileBlockMetaManager::new(batis.clone());

        ServiceContext {
            rb: batis.clone(),
            config,
            cloud_meta_manager: SimpleCloudMetaManager::new(batis.clone()),
            file_meta_manager: file_meta_manager.clone(),
            file_block_meta_manager: file_block_meta_manager.clone(),
            file_manager: SimpleFileManager::new(batis.clone(), file_meta_manager.clone(), file_block_meta_manager.clone()),
            config_manager: ConfigManager::new(batis.clone()),
            event_message_manager: EventMessageManager::new(batis.clone()),
            cloud_file_block_manager: CloudFileBlockManager::new(batis.clone()),
            user_manager: UserManager::new(batis.clone()),
        }
    }
}
