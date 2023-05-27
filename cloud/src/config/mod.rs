use log::info;

/// Config
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
}

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
