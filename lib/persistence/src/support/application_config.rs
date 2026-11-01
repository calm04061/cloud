use log::info;
// use rbatis::RBatis;

use crate::ApplicationConfig;

impl Default for ApplicationConfig {
    fn default() -> Self {
        let yml_data = include_str!("../../application.yml");
        //load config
        let mut result: ApplicationConfig =
            serde_yml::from_str(yml_data).expect("load config file fail");
        // DATABASE_URL 环境变量优先于编译内置的 application.yml，便于本地/测试环境切换数据库
        if let Ok(env_url) = std::env::var("DATABASE_URL") {
            if !env_url.is_empty() {
                info!("database_url override from env:{}", env_url);
                result.database_url = env_url;
            }
        }
        if result.debug {
            info!(" load config:{:?}", result);
            info!(" ///////////////////// Start On Debug Mode ////////////////////////////");
        } else {
            info!(" ///////////////////// Start On Release Mode ////////////////////////////");
        }
        result
    }
}

// pub fn init_rbatis(config: &ApplicationConfig) -> RBatis {
//     let rbatis = RBatis::new();
//     if rbatis.is_debug_mode() == false && config.debug.eq(&true) {
//         panic!(
//             r#"已使用release模式运行，但是仍使用debug模式！请修改 application.yml 中debug配置项为  debug: false"#
//         );
//     }
//
//     rbatis
// }