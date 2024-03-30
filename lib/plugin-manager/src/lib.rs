use api::{MetaInfo, Plugin, PluginMetaInfo};
use libloading::{Error, Library, Symbol};
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub static PLUGIN_MANAGER: Lazy<Arc<PluginManager>> = Lazy::new(|| Arc::new(PluginManager::new()));

pub struct PluginManager {
    plugins: Arc<Vec<PluginMetaInfo>>,
}

impl PluginManager {
    fn new() -> PluginManager {
        let plugins = Self::load_plugin();
        PluginManager { plugins: Arc::new(plugins) }
    }
    pub fn get_plugins(&self) -> Arc<Vec<PluginMetaInfo>> {
        Arc::clone(&self.plugins)
    }
    /// 获得目录对象
    /// 循环查找所有扩展
    /// 加载动态链接库
    /// 构建Plugin结构
    pub fn load_plugin() -> Vec<PluginMetaInfo> {
        let plugin_dir = Self::find_plugin_dir();
        if plugin_dir.is_none() {
            panic!("not found plugin dir");
        }
        let plugin_dir = plugin_dir.unwrap();
        let plugin_dir: &Path = plugin_dir.as_ref();
        let path_buf = plugin_dir.to_path_buf();
        let plugin_dir = path_buf.as_path();
        if plugin_dir.is_dir() {
            info!("use plugin dir: {}", plugin_dir.display());
        } else {
            warn!("plugin dir {} not found", plugin_dir.display());
            return vec![];
        }
        let result = plugin_dir.read_dir().unwrap();
        let plugin_extend_name = Self::plugin_extend();
        let plugin_extend_name = plugin_extend_name.as_str();
        let mut plugins = vec![];
        for file in result {
            let entry = file.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            let file_name = entry.file_name();
            let path_str = file_name.to_str().unwrap();
            let name = String::from(path_str);
            if name.ends_with(plugin_extend_name) {
                let buf = entry.path();
                let plugin_file = buf.to_str().unwrap();
                let option = Self::load_dynamic_plugin(plugin_file);
                if let Some(p) = option {
                    plugins.push(p);
                }
            }
        }
        plugins
    }

    /// 1. 参数
    /// 2. 环境变量
    /// 3. 配置文件
    /// 4. 默认值

    fn find_plugin_dir() -> Option<PathBuf> {
        let plugin_dir = env::var("PLUGIN_DIR");
        if let Ok(v) = plugin_dir {
            let plugin_dir: &Path = v.as_ref();
            let plugin_dir = plugin_dir.to_path_buf();
            return Some(plugin_dir);
        }
        let result = env::current_dir();
        if result.is_err() {
            return None;
        }
        info!("not found plugin dir use default plugin dir ./plugin");
        Some("./plugin".into())
    }

    fn load_dynamic_plugin(plugin_file: &str) -> Option<PluginMetaInfo> {
        unsafe {
            info!("loading plugin: {}", plugin_file);
            let lib = Library::new(plugin_file);
            if let Err(e) = lib {
                error!("load plugin: {} error:{}", plugin_file, e);
                return None;
            }
            let library = lib.unwrap();
            let meta_info: Result<Symbol<unsafe extern fn() -> Box<dyn Plugin>>, Error> = library.get(b"plugin_meta");
            if meta_info.is_err() {
                error!("{},not a plugin", plugin_file);
                return None;
            }
            let meta_info = meta_info.unwrap();
            let meta_info = meta_info();
            let name = meta_info.name();
            let version = meta_info.version();
            let capacities = meta_info.capacities();
            let meta_info = MetaInfo {
                name: name.to_string(),
                version: version.to_string(),
                capacities,
            };

            // let meta_info: MetaInfo = meta_info.into();
            info!("load plugin: {}@{} success", meta_info.name.clone(),meta_info.version.clone());
            Some(PluginMetaInfo {
                meta_info,
                library,
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn plugin_extend() -> String {
        ".dll".to_string()
    }

    #[cfg(target_os = "linux")]
    fn plugin_extend() -> String {
        ".so".to_string()
    }

    #[cfg(target_os = "macos")]
    fn plugin_extend() -> String {
        ".dylib".to_string()
    }
}