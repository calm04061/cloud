use api::{Capacity, MetaInfo, WasiPlugin};
use log::{error, info, warn};
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::*;

#[derive(Clone)]
pub struct WebResource {
    pub path: String,
    pub content_type: String,
}

pub struct WasiWebInstance {
    name: String,
    version: String,
    capacities: Vec<Capacity>,
    instance: Instance,
    store: Mutex<Store<()>>,
    web_resources: Vec<WebResource>,
}

unsafe impl Send for WasiWebInstance {}
unsafe impl Sync for WasiWebInstance {}

impl WasiWebInstance {
    pub fn load(wasm_path: &Path) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[])?;

        let name_func = instance.get_typed_func::<(), i32>(&mut store, "name")?;
        let version_func = instance.get_typed_func::<(), i32>(&mut store, "version")?;
        let web_config_func = instance.get_typed_func::<(), i32>(&mut store, "web_config")?;

        let name_ptr = name_func.call(&mut store, ())?;
        let version_ptr = version_func.call(&mut store, ())?;
        let config_ptr = web_config_func.call(&mut store, ())?;

        let name = Self::read_string(&instance, &mut store, name_ptr)?;
        let version = Self::read_string(&instance, &mut store, version_ptr)?;
        let config_json = Self::read_string(&instance, &mut store, config_ptr)?;

        let capacities: Vec<Capacity> = serde_json::from_str(&config_json)
            .unwrap_or_else(|_| vec![Capacity::WEB("/".to_string())]);

        let web_resources = capacities
            .iter()
            .filter_map(|c| match c {
                Capacity::WEB(path) => Some(WebResource {
                    path: path.clone(),
                    content_type: "text/html".to_string(),
                }),
                _ => None,
            })
            .collect();

        info!("loaded WASI web plugin: {}@{}", name, version);

        Ok(WasiWebInstance {
            name,
            version,
            capacities,
            instance,
            store: Mutex::new(store),
            web_resources,
        })
    }

    fn read_string(instance: &Instance, store: &mut Store<()>, ptr: i32) -> anyhow::Result<String> {
        if ptr == 0 {
            return Ok(String::new());
        }
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("failed to get memory"))?;
        let data = memory.data(&*store);
        let len = data.len() as i32;
        if ptr < 0 || ptr >= len {
            return Ok(String::new());
        }
        let mut end = ptr;
        while end < len && data[end as usize] != 0 {
            end += 1;
        }
        let slice = &data[ptr as usize..end as usize];
        Ok(String::from_utf8_lossy(slice).to_string())
    }
}

impl WasiPlugin for WasiWebInstance {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn capacities(&self) -> Vec<Capacity> {
        self.capacities.clone()
    }

    fn call(&self, _method: &str, _params: &str) -> Result<String, String> {
        Err("web plugin does not support call method".to_string())
    }
}

pub struct WasiWebManager {
    plugins: Vec<WasiWebInstance>,
}

impl WasiWebManager {
    pub fn new() -> Self {
        let plugins = Self::load_plugins();
        WasiWebManager { plugins }
    }

    pub fn load_plugins() -> Vec<WasiWebInstance> {
        let plugin_dir = Self::find_plugin_dir();
        if plugin_dir.is_none() {
            warn!("WASI web plugin directory not found");
            return vec![];
        }
        let plugin_dir = plugin_dir.unwrap();
        let plugin_dir: &Path = plugin_dir.as_ref();

        if !plugin_dir.is_dir() {
            warn!("WASI web plugin dir {} not found", plugin_dir.display());
            return vec![];
        }

        info!("loading WASI web plugins from: {}", plugin_dir.display());

        let mut plugins = vec![];
        if let Ok(entries) = plugin_dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "wasm") {
                    match WasiWebInstance::load(&path) {
                        Ok(plugin) => plugins.push(plugin),
                        Err(e) => error!("failed to load WASI web plugin {:?}: {}", path, e),
                    }
                }
            }
        }
        plugins
    }

    fn find_plugin_dir() -> Option<std::path::PathBuf> {
        if let Ok(dir) = std::env::var("WASI_WEB_PLUGIN_DIR") {
            return Some(dir.into());
        }
        if let Ok(dir) = std::env::var("WASI_PLUGIN_DIR") {
            return Some(dir.into());
        }
        Some("./wasi-plugins".into())
    }

    pub fn get_plugins(&self) -> Vec<api::PluginMetaInfo> {
        self.plugins
            .iter()
            .map(|p| api::PluginMetaInfo {
                meta_info: MetaInfo {
                    name: p.name(),
                    version: p.version(),
                    capacities: p.capacities(),
                },
                instance: api::PluginInstance::Wasi(Arc::new(p.clone())),
            })
            .collect()
    }

    pub fn get_web_resources(&self) -> Vec<WebResource> {
        let mut resources = vec![];
        for plugin in &self.plugins {
            resources.extend(plugin.web_resources.clone());
        }
        resources
    }
}

impl Default for WasiWebManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WasiWebInstance {
    fn clone(&self) -> Self {
        let store = self.store.lock().unwrap();
        WasiWebInstance {
            name: self.name.clone(),
            version: self.version.clone(),
            capacities: self.capacities.clone(),
            instance: self.instance.clone(),
            store: Mutex::new(Store::new(store.engine(), ())),
            web_resources: self.web_resources.clone(),
        }
    }
}
