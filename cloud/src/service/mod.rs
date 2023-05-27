use crate::config::ApplicationConfig;
use crate::database::config::ConfigManager;
use crate::database::meta::cloud::SimpleCloudMetaManager;
use crate::database::meta::file::file_block_meta::SimpleFileBlockMetaManager;
use crate::database::meta::file::file_meta::SimpleFileMetaManager;
use crate::database::meta::file::SimpleFileManager;
use once_cell::sync::Lazy;
use rbatis::Rbatis;

pub(crate) static CONTEXT: Lazy<ServiceContext> = Lazy::new(|| ServiceContext::default());
#[macro_export]
macro_rules! pool {
    () => {
        &mut $crate::service::CONTEXT.rb.clone()
    };
}
pub(crate) struct ServiceContext {
    pub config: ApplicationConfig,
    pub rb: Rbatis,
    pub cloud_meta_manager: SimpleCloudMetaManager,
    pub file_meta_manager: SimpleFileMetaManager,
    pub file_block_meta_manager: SimpleFileBlockMetaManager,
    pub file_manager: SimpleFileManager,
    pub config_manager: ConfigManager,
}

impl ServiceContext {
    /// init database pool
    pub async fn init_pool(&self) {
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
            self.rb.get_pool().expect("pool not init!").status()
        );
    }
}

impl Default for ServiceContext {
    fn default() -> Self {
        let config = ApplicationConfig::default();

        ServiceContext {
            rb: crate::domain::init_rbatis(&config),
            config,
            cloud_meta_manager: SimpleCloudMetaManager::new(),
            file_meta_manager: SimpleFileMetaManager::new(),
            file_block_meta_manager: SimpleFileBlockMetaManager::new(),
            file_manager: SimpleFileManager::new(),
            config_manager: ConfigManager::new(),
        }
    }
}
