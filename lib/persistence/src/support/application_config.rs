use log::info;
use rbatis::RBatis;

use crate::ApplicationConfig;

impl Default for ApplicationConfig {
    fn default() -> Self {
        let yml_data = include_str!("../../application.yml");
        //load config
        let result: ApplicationConfig =
            serde_yaml::from_str(yml_data).expect("load config file fail");
        if result.debug {
            info!(" load config:{:?}", result);
            info!(" ///////////////////// Start On Debug Mode ////////////////////////////");
        } else {
            info!(" ///////////////////// Start On Release Mode ////////////////////////////");
        }
        result
    }
}

pub fn init_rbatis(config: &ApplicationConfig) -> RBatis {
    let rbatis = RBatis::new();
    if rbatis.is_debug_mode() == false && config.debug.eq(&true) {
        panic!(
            r#"已使用release模式运行，但是仍使用debug模式！请修改 application.yml 中debug配置项为  debug: false"#
        );
    }

    rbatis
}