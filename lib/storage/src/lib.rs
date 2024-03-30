use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::storage_facade::StorageFacade;

pub(crate) mod ali;
pub(crate) mod baidu;
pub(crate) mod local;
pub mod storage;
pub mod storage_facade;
pub(crate) mod onedrive;
#[cfg(not(windows))]
pub(crate) mod sftp;
pub mod web;
pub mod model;

pub static STORAGE_FACADE: Lazy<Arc<RwLock<StorageFacade>>> = Lazy::new(|| Arc::new(RwLock::new(StorageFacade::new())));
