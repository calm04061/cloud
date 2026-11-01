use chrono::{Local, TimeZone};
use log::info;
use nfsserve::nfs::fileid3;

use api::error::ErrorInfo;
use api::{ResponseResult, ROOT_FILE_ID};
use persistence::meta::FileMeta;
use persistence::FileMetaType;
use service::meta::FileManager;
use service::CONTEXT;

use super::VirtualFileSystem;

/// 命名空间职责：路径解析与文件元数据操作，只触碰 file_manager。
impl VirtualFileSystem {
    pub(crate) async fn path_meta(&self, path: &str) -> ResponseResult<FileMeta> {
        let mut parent: Option<FileMeta> = None;
        if path.eq("/") {
            let temp_parent = CONTEXT
                .file_manager
                .info_by_id(1)
                .await;
            parent = match temp_parent {
                Ok(meta) => Some(meta),
                Err(_e) => None,
            }
        } else {
            let split = path.split("/");

            for name in split {
                let parent_id;
                if let None = parent {
                    parent_id = 0;
                } else {
                    parent_id = parent.unwrap().id.unwrap();
                }
                let option;
                if parent_id == 0 && name.eq("") {
                    option = Some(CONTEXT
                        .file_manager
                        .info_by_id(1)
                        .await?);
                } else {
                    option = Some(CONTEXT
                        .file_manager
                        .info_by_parent_and_name(parent_id, name)
                        .await?);
                }
                if let None = option {
                    return Err(ErrorInfo::FileNotFound(name.to_string()));
                }
                parent = option;
            }
        }
        Ok(parent.unwrap())
    }
    pub(crate) async fn path_info(&self, path: &str) -> ResponseResult<(FileMeta, String)> {
        let split = path.split("/");
        let name = split.last().unwrap();
        let end = (path.len() - name.len()) - 1;
        let parent_path = &path[0..end];
        let parent: FileMeta = self.path_meta(parent_path).await?;
        Ok((parent, name.to_string()))
    }
    pub(crate) async fn create_file(
        &self,
        parent: u64,
        name: &str,
        file_type: FileMetaType,
    ) -> ResponseResult<FileMeta> {
        let option = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent as i64, name)
            .await;
        if let Ok(_) = option {
            return Err(ErrorInfo::FileAlreadyExist(name.to_string()));
        }
        CONTEXT
            .file_manager
            .new_file(parent as i64, name, file_type)
            .await
    }
    pub(crate) async fn create_dir(&self, path: &str) -> ResponseResult<FileMeta> {
        let (parent, name) = self.path_info(path).await?;
        let parent_id = parent.id.ok_or_else(|| ErrorInfo::new(0, "parent id missing"))?;
        CONTEXT
            .file_manager
            .new_file(parent_id, name.as_str(), FileMetaType::DIR)
            .await
    }
    pub(crate) async fn create_path_file(
        &self,
        path: &str,
    ) -> ResponseResult<FileMeta> {
        let (meta, name) = self.path_info(path).await?;
        let parent_id = meta.id.unwrap();
        CONTEXT
            .file_manager
            .new_file(parent_id, name.as_str(), FileMetaType::FILE)
            .await
    }
    pub(crate) async fn del_file(&self, parent: u64, name: &str) -> ResponseResult<()> {
        info!("del_file {:?}@{}", name, parent);
        let option = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent as i64, name)
            .await;
        if let Err(_e) = option {
            return Err(ErrorInfo::FileNotFound(name.to_string()));
        }
        let meta = option?;
        CONTEXT
            .file_manager
            .delete_file_meta(meta.id.unwrap())
            .await?;
        Ok(())
    }
    pub(crate) async fn delete_one_file(&self, path: &str) -> ResponseResult<()> {
        let file_meta = self.path_meta(path).await?;
        CONTEXT
            .file_manager
            .delete_one_file_meta(file_meta.id.unwrap())
            .await?;
        Ok(())
    }
    pub(crate) async fn rename(
        &self,
        parent: u64,
        name: &str,
        new_parent: u64,
        new_name: &str,
    ) -> ResponseResult<()> {
        let source_file = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent as i64, name)
            .await;
        let mut source_file = match source_file {
            Ok(f) => {
                f
            }
            Err(_e) => {
                return Err(ErrorInfo::FileNotFound("源文件不存在".to_string()));
            }
        };

        let target_file = CONTEXT
            .file_manager
            .info_by_parent_and_name(new_parent as i64, new_name)
            .await;
        if let Ok(_e) = target_file {
            return Err(ErrorInfo::FileAlreadyExist("目标文件已经存在".to_string()));
        }

        source_file.parent_id = new_parent as i64;
        source_file.name = String::from(new_name);
        CONTEXT.file_manager.update_meta(source_file).await?;
        Ok(())
    }
    pub(crate) async fn rename_path(&self, from_full_path: &str, to_full_path: &str) -> ResponseResult<()> {
        let (parent_from, name_from) = self.path_info(from_full_path).await?;
        let (parent_to, name_to) = self.path_info(to_full_path).await?;
        self.rename(parent_from.id.unwrap() as u64, name_from.as_str(), parent_to.id.unwrap() as u64, name_to.as_str()).await
    }

    pub(crate) async fn lookup(&self, parent: u64, name: &str) -> ResponseResult<FileMeta> {
        let result = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent as i64, name)
            .await;
        if let Ok(f) = result {
            return Ok(f);
        }
        Err(ErrorInfo::new_string(3, format!("文件[{}]不存在", name)))
    }
    pub(crate) async fn file_info(&self, id: u64) -> ResponseResult<FileMeta> {
        CONTEXT.file_manager.info_by_id(id as i64).await
    }
    pub(crate) async fn update_file_meta(&self, f: FileMeta) -> ResponseResult<FileMeta> {
        CONTEXT.file_manager.update_meta(f).await
    }
    pub(crate) async fn file_info_by_path(&self, path: &str) -> ResponseResult<FileMeta> {
        if path.eq("") || path.eq("/") {
            return CONTEXT.file_manager.info_by_id(ROOT_FILE_ID as i64).await;
        }
        let mut temp_path = path;
        if path.starts_with("/") {
            temp_path = &path[1..];
        }
        let split = temp_path.split("/");
        let mut file: Option<FileMeta> = None;
        for name in split {
            if let None = file {
                file = Some(CONTEXT.file_manager.info_by_parent_and_name(ROOT_FILE_ID as i64, name).await?);
                continue;
            }
            let meta = file.unwrap();
            file = Some(CONTEXT.file_manager.info_by_parent_and_name(meta.id.unwrap(), name).await?);
        }
        Ok(file.unwrap())
    }

    pub(crate) async fn list_by_parent(&self, ino: u64) -> ResponseResult<Vec<FileMeta>> {
        CONTEXT.file_manager.list_by_parent(ino as i64).await
    }
    pub(crate) async fn list_by_parent_page(&self, dirid: fileid3, ino: u64, max_entries: usize) -> ResponseResult<(Vec<FileMeta>, bool)> {
        CONTEXT.file_manager.list_by_parent_page(dirid as i64, ino as i64, max_entries).await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn setattr(
        &self,
        ino: u64,
        size: Option<u64>,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        mtime_millis: Option<i64>,
    ) -> ResponseResult<FileMeta> {
        let result = CONTEXT.file_manager.info_by_id(ino as i64).await;
        if let Err(e) = result {
            return Err(e);
        }
        let mut f = result?;

        if let Some(size_value) = size {
            f.file_length = size_value as i64;
        }
        if let Some(mode_value) = mode {
            f.mode = mode_value as i32;
        }
        if let Some(uid_value) = uid {
            f.uid = uid_value as i32;
        }
        if let Some(gid_value) = gid {
            f.gid = gid_value as i32;
        }
        if let Some(mtime_value) = mtime_millis {
            if let Some(time) = Local.timestamp_millis_opt(mtime_value).single() {
                f.update_time = time;
            }
        }
        CONTEXT.file_manager.update_meta(f).await
    }
}
