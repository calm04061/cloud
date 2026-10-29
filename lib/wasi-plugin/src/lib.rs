use api::{Capacity, MetaInfo, WasiPlugin};
use log::{error, info, warn};
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::*;

pub struct WasiPluginInstance {
    name: String,
    version: String,
    capacities: Vec<Capacity>,
    instance: Instance,
    store: Mutex<Store<()>>,
}

unsafe impl Send for WasiPluginInstance {}
unsafe impl Sync for WasiPluginInstance {}

impl WasiPluginInstance {
    pub fn load(wasm_path: &Path) -> anyhow::Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| anyhow::anyhow!("failed to parse WebAssembly module: {} - {:?}", e, e))?;
        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| anyhow::anyhow!("failed to instantiate module: {}", e))?;

        let name_func = instance.get_typed_func::<(), i32>(&mut store, "name")?;
        let version_func = instance.get_typed_func::<(), i32>(&mut store, "version")?;
        let capacities_func = instance.get_typed_func::<(), i32>(&mut store, "capacities")?;

        let name_ptr = name_func.call(&mut store, ())?;
        let version_ptr = version_func.call(&mut store, ())?;
        let capacities_ptr = capacities_func.call(&mut store, ())?;

        let name = Self::read_string(&instance, &mut store, name_ptr)?;
        let version = Self::read_string(&instance, &mut store, version_ptr)?;
        let capacities_json = Self::read_string(&instance, &mut store, capacities_ptr)?;
        let capacities: Vec<Capacity> = serde_json::from_str(&capacities_json).unwrap_or_default();

        info!("loaded WASI plugin: {}@{}", name, version);

        Ok(WasiPluginInstance {
            name,
            version,
            capacities,
            instance,
            store: Mutex::new(store),
        })
    }

    fn read_string(instance: &Instance, store: &mut Store<()>, ptr: i32) -> anyhow::Result<String> {
        if ptr == 0 {
            return Ok(String::new());
        }
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("failed to get memory from WASI plugin"))?;
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

impl WasiPlugin for WasiPluginInstance {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn version(&self) -> String {
        self.version.clone()
    }

    fn capacities(&self) -> Vec<Capacity> {
        self.capacities.clone()
    }

    fn call(&self, method: &str, params: &str) -> Result<String, String> {
        let func_name = format!("call_{}", method);

        let mut store = self.store.lock().map_err(|e| e.to_string())?;

        if let Ok(func) = self
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut *store, &func_name)
        {
            let alloc_func = self
                .instance
                .get_typed_func::<(i32, i32), i32>(&mut *store, "alloc")
                .map_err(|e| e.to_string())?;

            let method_ptr = alloc_func
                .call(&mut *store, (method.as_ptr() as i32, method.len() as i32))
                .map_err(|e| e.to_string())?;
            let params_ptr = alloc_func
                .call(&mut *store, (params.as_ptr() as i32, params.len() as i32))
                .map_err(|e| e.to_string())?;

            let result = func
                .call(&mut *store, (method_ptr, params_ptr))
                .map_err(|e| e.to_string())?;
            let response = Self::read_string(&self.instance, &mut *store, result)
                .map_err(|e| e.to_string())?;
            Ok(response)
        } else {
            Err(format!("method {} not found", method))
        }
    }
}

pub struct WasiPluginManager {
    plugins: Vec<WasiPluginInstance>,
}

impl WasiPluginManager {
    pub fn new() -> Self {
        let plugins = Self::load_plugins();
        WasiPluginManager { plugins }
    }

    pub fn load_plugins() -> Vec<WasiPluginInstance> {
        let plugin_dir = Self::find_plugin_dir();
        if plugin_dir.is_none() {
            warn!("WASI plugin directory not found");
            return vec![];
        }
        let plugin_dir = plugin_dir.unwrap();
        let plugin_dir: &Path = plugin_dir.as_ref();

        if !plugin_dir.is_dir() {
            warn!("WASI plugin dir {} not found", plugin_dir.display());
            return vec![];
        }

        info!("loading WASI plugins from: {}", plugin_dir.display());

        let mut plugins = vec![];
        if let Ok(entries) = plugin_dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "wasm") {
                    match WasiPluginInstance::load(&path) {
                        Ok(plugin) => plugins.push(plugin),
                        Err(e) => error!("failed to load WASI plugin {:?}: {}", path, e),
                    }
                }
            }
        }
        plugins
    }

    fn find_plugin_dir() -> Option<std::path::PathBuf> {
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
}

impl Clone for WasiPluginInstance {
    fn clone(&self) -> Self {
        let store = self.store.lock().unwrap();
        WasiPluginInstance {
            name: self.name.clone(),
            version: self.version.clone(),
            capacities: self.capacities.clone(),
            instance: self.instance.clone(),
            store: Mutex::new(Store::new(store.engine(), ())),
        }
    }
}
