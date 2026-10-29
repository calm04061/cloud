use once_cell::sync::Lazy;

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
mod google;

pub static STORAGE_FACADE: Lazy<StorageFacade> = Lazy::new(StorageFacade::new);
