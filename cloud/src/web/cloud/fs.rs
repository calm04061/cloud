use std::ffi::OsString;
use std::sync::Mutex;

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, Responder, Result};
use fuser::MountOption::{AllowOther, AutoUnmount};
use fuser::{BackgroundSession, MountOption};
use log::info;

use crate::database::meta::CloudMetaManager;
use crate::domain::table::tables::Config;
use crate::fs::fuse::cloud_fs::CloudFS;
use crate::fs::vfs::DEFAULT_TEMP_PATH;
use crate::service::CONTEXT;
use crate::web::common::WebResult;

const MOUNT_PATH: &str = "MOUNT_PATH";
pub(crate) const TEMP_PATH: &str = "TEMP_PATH";

pub(crate) struct FsManager {
    session_holder: Mutex<Option<BackgroundSession>>,
}

impl FsManager {
    pub(crate) fn new() -> FsManager {
        FsManager {
            session_holder: Mutex::new(None),
        }
    }
}

unsafe impl Send for FsManager {}

impl Drop for FsManager {
    fn drop(&mut self) {
        let mut guard = self.session_holder.lock().unwrap();
        let option = guard.take();
        if let Some(mut _se) = option {}
        info!("fs manager dropping")
    }
}

#[get("/fs/mount")]
pub(crate) async fn mount(fs_manager: Data<FsManager>) -> Result<impl Responder> {
    let mount_path_config = CONTEXT.config_manager.info(String::from(MOUNT_PATH)).await;
    if let None = mount_path_config {
        return Ok(Json(WebResult::<Config>::fail(1, "没有配置挂在路径")));
    }
    let mount_path = mount_path_config.clone().unwrap();
    let temp_path_config = CONTEXT.config_manager.info(String::from(TEMP_PATH)).await;
    let cache_file = match temp_path_config {
        None => String::from(DEFAULT_TEMP_PATH),
        Some(config) => config.value.clone(),
    };
    let options = vec![
        MountOption::FSName("CloudFs".to_string()),
        AutoUnmount,
        AllowOther,
    ];
    let cloud_fs = CloudFS::from(cache_file.as_str());
    let mount_point = OsString::from(mount_path.value.as_str());
    let result = fuser::spawn_mount2(cloud_fs, mount_point, &options);
    // let result = Session::new(cloud_fs, mount_point.as_ref(), &options);
    match result {
        Ok(se) => {
            let mut result1 = fs_manager.session_holder.lock().unwrap();
            let _x = result1.insert(se);
            Ok(Json(WebResult::empty()))
        }
        Err(e) => {
            let msg = format!("{}", e);
            Ok(Json(WebResult::fail(4, msg.as_str())))
        }
    }
}

#[delete("/fs/mount/{id}")]
pub(crate) async fn umount(id: Path<i32>) -> Result<impl Responder> {
    let x = CONTEXT.cloud_meta_manager.info(id.into_inner()).await;
    Ok(WebResult::actix_web_json_result(&x))
}
