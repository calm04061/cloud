use api::{MetaInfo, Plugin, PluginInstance, PluginMetaInfo, PLUGIN_ABI_VERSION};
use libloading::{Error, Library, Symbol};
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasi_plugin::WasiPluginManager;

pub static PLUGIN_MANAGER: Lazy<Arc<PluginManager>> = Lazy::new(|| Arc::new(PluginManager::new()));

pub struct PluginManager {
    plugins: Arc<Vec<PluginMetaInfo>>,
}

impl PluginManager {
    fn new() -> PluginManager {
        let native_plugins = Self::load_native_plugin();
        let wasi_plugins = Self::load_wasi_plugin();
        let mut all_plugins = native_plugins;
        all_plugins.extend(wasi_plugins);
        PluginManager {
            plugins: Arc::new(all_plugins),
        }
    }

    pub fn get_plugins(&self) -> Arc<Vec<PluginMetaInfo>> {
        Arc::clone(&self.plugins)
    }

    fn load_wasi_plugin() -> Vec<PluginMetaInfo> {
        let manager = WasiPluginManager::new();
        manager.get_plugins()
    }

    pub fn load_native_plugin() -> Vec<PluginMetaInfo> {
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

    fn plugin_allowed(name: &str) -> bool {
        let allowlist = env::var("PLUGIN_ALLOWLIST").unwrap_or_default();
        if allowlist.trim().is_empty() {
            warn!("PLUGIN_ALLOWLIST is empty; all plugins are treated as trusted");
            return true;
        }
        allowlist
            .split(',')
            .map(str::trim)
            .any(|item| item == name)
    }

    fn plugin_abi_supported(library: &Library, plugin_file: &str) -> bool {
        unsafe {
            let abi_version: Result<Symbol<unsafe extern "C" fn() -> u32>, Error> =
                library.get(b"plugin_abi_version");
            match abi_version {
                Ok(symbol) => {
                    let version = symbol();
                    if version != PLUGIN_ABI_VERSION {
                        error!(
                            "plugin {} abi version {} is not supported",
                            plugin_file, version
                        );
                        return false;
                    }
                    true
                }
                Err(_) => {
                    warn!(
                        "plugin {} does not export plugin_abi_version; treating as legacy",
                        plugin_file
                    );
                    true
                }
            }
        }
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
            let meta_info: Result<Symbol<unsafe extern "C" fn() -> Box<dyn Plugin>>, Error> =
                library.get(b"plugin_meta");
            if meta_info.is_err() {
                error!("{},not a plugin", plugin_file);
                return None;
            }
            let meta_info = meta_info.unwrap();
            let meta_info = meta_info();
            let name = meta_info.name();
            let version = meta_info.version();
            let capacities = meta_info.capacities();

            if !Self::plugin_allowed(&name) {
                warn!("plugin {} is not in PLUGIN_ALLOWLIST", name);
                return None;
            }
            if !Self::plugin_abi_supported(&library, plugin_file) {
                return None;
            }

            let meta_info = MetaInfo {
                name: name.to_string(),
                version: version.to_string(),
                capacities,
            };

            info!(
                "load plugin: {}@{} success",
                meta_info.name.clone(),
                meta_info.version.clone()
            );
            Some(PluginMetaInfo {
                meta_info,
                instance: PluginInstance::Native(Arc::new(library)),
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
