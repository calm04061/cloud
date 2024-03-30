use std::fmt::{Debug, Formatter};
use std::fs;
use std::fs::File;
use std::io::SeekFrom::Start;
use std::io::{Read, Seek, Write};
use std::path::Path;

use crypto::digest::Digest;
use crypto::md5::Md5;
use log::{debug, error, info};
use nfsserve::nfs::fileid3;

use api::error::ErrorInfo;
use api::{ResponseResult, ROOT_FILE_ID};
use persistence::meta::{FileBlockMeta, FileMeta};
use persistence::{FileMetaType, FileStatus};
use service::meta::FileManager;
use service::CONTEXT;
use storage::STORAGE_FACADE;

pub const DEFAULT_TEMP_PATH: &str = "/var/lib/storage/temp";

pub const CLOUD_FILE_BLOCK_SIZE: usize = 4194304 * 4;

// 16384 * 256; //1024k
#[derive(Clone, Debug)]
pub(crate) struct VirtualFileSystem {
    inner: Inner,
}

#[derive(Clone)]
struct Inner {
    pub cache_file: String,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.cache_file)
            .finish()
    }
}

impl VirtualFileSystem {
    pub(crate) fn new(cache_file: &str) -> VirtualFileSystem {
        VirtualFileSystem {
            inner: Inner {
                cache_file: String::from(cache_file),
            },
        }
    }
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
            .info_by_parent_and_name(parent, name)
            .await;
        if let Ok(_) = option {
            return Err(ErrorInfo::FileAlreadyExist(name.to_string()));
        }
        CONTEXT
            .file_manager
            .new_file(parent, name, file_type)
            .await
    }
    pub(crate) async fn create_dir(&self, path: &str) -> ResponseResult<FileMeta> {
        let (parent, name) = self.path_info(path).await?;
        let option = CONTEXT
            .file_manager
            .new_file(parent.id.unwrap(), name.as_str(), FileMetaType::DIR)
            .await.ok().unwrap();

        Ok(option)
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
    pub(crate) fn del_file(&self, parent: u64, name: &str) -> ResponseResult<()> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async {
                self.del_file_sync(parent, name).await
            })
    }
    pub(crate) async fn del_file_sync(&self, parent: u64, name: &str) -> ResponseResult<()> {
        info!("del_file {:?}@{}", name, parent);
        let option = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent, name)
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

    // pub(crate) async fn delete_file(&self, path: &str) -> ResponseResult<()> {
    //     let file_meta = self.path_meta(path).await?;
    //     CONTEXT
    //         .file_manager
    //         .delete_file_meta(file_meta.id.unwrap())
    //         .await?;
    //     Ok(())
    // }
    pub(crate) async fn delete_one_file(&self, path: &str) -> ResponseResult<()> {
        let file_meta = self.path_meta(path).await?;
        CONTEXT
            .file_manager
            .delete_one_file_meta(file_meta.id.unwrap())
            .await?;
        Ok(())
    }

    ///
    /// 循环写入多个文件块
    ///
    pub(crate) async fn write(&mut self, ino: u64, offset: u64, data: &[u8]) -> ResponseResult<u32> {
        let mut start = 0;
        loop {
            let temp = &data[start..];
            let result = self
                .inner
                .write(ino, start as u64 + offset, temp)
                .await?;
            start = start + result as usize;
            if start == data.len() {
                break;
            }
        }
        Ok(data.len() as u32)
    }

    async fn read_content(
        &self,
        file_block_metas: Vec<FileBlockMeta>,
        offset: u64,
        size: u32,
    ) -> ResponseResult<Vec<u8>> {
        let mut temp_body = Vec::new();
        let start_block_index = offset / CLOUD_FILE_BLOCK_SIZE as u64;
        let mut seek = offset % CLOUD_FILE_BLOCK_SIZE as u64;
        for file_block_meta in file_block_metas {
            if file_block_meta.block_index < start_block_index as i64 {
                continue;
            }
            let mut body = self.inner.read_block(&file_block_meta, seek).await;
            seek = 0;
            if let Err(e) = body {
                if let ErrorInfo::Retry = e {
                    body = self.inner.read_block(&file_block_meta, seek).await;
                } else {
                    return Err(e);
                }
            }
            let body = body?;
            let vec = body.to_vec();
            for a in vec {
                temp_body.push(a);
                if temp_body.len() == size as usize {
                    return Ok(temp_body);
                }
            }
        }
        Ok(temp_body)
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
            .info_by_parent_and_name(parent, name)
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
            .info_by_parent_and_name(new_parent, new_name)
            .await;
        if let Ok(_e) = target_file {
            return Err(ErrorInfo::FileAlreadyExist("目标文件已经存在".to_string()));
        }

        source_file.parent_id = new_parent;
        source_file.name = String::from(new_name);
        CONTEXT.file_manager.update_meta(source_file).await?;
        Ok(())
    }
    pub(crate) async fn rename_path(&self, from_full_path: &str, to_full_path: &str) -> ResponseResult<()> {
        let (parent_from, name_from) = self.path_info(from_full_path).await?;
        let (parent_to, name_to) = self.path_info(to_full_path).await?;
        self.rename(parent_from.id.unwrap(), name_from.as_str(), parent_to.id.unwrap(), name_to.as_str()).await.ok();
        Ok(())
    }


    pub(crate) fn lookup(&self, parent: u64, name: &str) -> ResponseResult<FileMeta> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async {
                self.lookup_sync(parent, name).await
            })
    }
    pub(crate) async fn lookup_sync(&self, parent: u64, name: &str) -> ResponseResult<FileMeta> {
        let result = CONTEXT
            .file_manager
            .info_by_parent_and_name(parent, name)
            .await;
        if let Ok(f) = result {
            return Ok(f);
        }
        Err(ErrorInfo::new_string(3, format!("文件[{}]不存在", name)))
    }
    pub(crate) fn file_info(&self, id: u64) -> ResponseResult<FileMeta> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async { self.file_info_sync(id).await })
    }
    pub(crate) async fn file_info_sync(&self, id: u64) -> ResponseResult<FileMeta> {
        CONTEXT.file_manager.info_by_id(id).await
    }
    pub(crate) async fn update_file_meta(&self, f: FileMeta) -> ResponseResult<FileMeta> {
        CONTEXT.file_manager.update_meta(f).await
    }
    pub(crate) async fn file_info_by_path(&self, path: &str) -> ResponseResult<FileMeta> {
        if path.eq("") || path.eq("/") {
            return CONTEXT.file_manager.info_by_id(ROOT_FILE_ID).await;
        }
        let mut temp_path = path;
        if path.starts_with("/") {
            temp_path = &path[1..];
        }
        let split = temp_path.split("/");
        let mut file: Option<FileMeta> = None;
        for name in split {
            if let None = file {
                file = Some(CONTEXT.file_manager.info_by_parent_and_name(ROOT_FILE_ID, name).await?);
                continue;
            }
            let meta = file.unwrap();
            file = Some(CONTEXT.file_manager.info_by_parent_and_name(meta.id.unwrap(), name).await?);
        }
        Ok(file.unwrap())
    }

    pub(crate) async fn read(&self, ino: u64, offset: u64, size: u32) -> ResponseResult<Vec<u8>> {
        debug!("read file id {},from {:?}:size={}", ino, offset, size);
        let result = CONTEXT.file_manager.info_by_id(ino).await;
        if let Err(e) = result {
            error!("查询文件{}失败{}", ino, e);
            return Err(e);
        }
        let f = result?;
        if f.file_length == 0 {
            return Ok(vec![]);
        }
        let file_block_metas = CONTEXT.file_manager.file_block_meta(f.id.unwrap()).await?;
        if file_block_metas.is_empty() {
            return Err(ErrorInfo::new(3, "文件块数据不存在"));
        }
        self.read_content(file_block_metas, offset, size).await
    }
    pub(crate) async fn list_by_parent(&self, ino: u64) -> ResponseResult<Vec<FileMeta>> {
        CONTEXT.file_manager.list_by_parent(ino).await
    }
    pub(crate) async fn list_by_parent_page(&self, dirid: fileid3, ino: u64, max_entries: usize) -> ResponseResult<(Vec<FileMeta>, bool)> {
        CONTEXT.file_manager.list_by_parent_page(dirid, ino, max_entries).await
    }

    pub(crate) fn setattr(
        &mut self,
        ino: u64,
        size: Option<u64>,
    ) -> ResponseResult<FileMeta> {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async {
                let result = CONTEXT.file_manager.info_by_id(ino).await;
                if let Err(e) = result {
                    return Err(e);
                }
                let mut f = result?;

                if let Some(size_value) = size {
                    f.file_length = size_value;
                }
                CONTEXT.file_manager.update_meta(f).await
            })
    }
}

impl Inner {
    fn read_local_file(&self, file_path: String, seek: u64) -> ResponseResult<Vec<u8>> {
        let result = File::open(file_path);

        if let Err(_e) = result {
            return Err(ErrorInfo::new(10, "文件不存在"));
        }
        let mut f = result?;
        let mut body: Vec<u8> = Vec::new();
        let result = f.seek(Start(seek));
        match result {
            Ok(_) => {
                debug!("跳过成功")
            }
            Err(_) => {
                return Err(ErrorInfo::new(12, "读取文件出错"));
            }
        }
        let result = f.read_to_end(&mut body);
        match result {
            Ok(_) => Ok(body),
            Err(_) => Err(ErrorInfo::new(11, "读取文件出错")),
        }
    }
    ///
    ///  读取文件块
    ///
    ///
    async fn read_block(
        &self,
        file_block_meta: &FileBlockMeta,
        seek: u64,
    ) -> ResponseResult<Vec<u8>> {
        let local_cache_file = format!("{}/{}", self.cache_file, file_block_meta.file_part_id);
        // 本地已尽存在直接返回
        let result = self.read_local_file(local_cache_file.clone(), seek);
        if let Ok(body) = result {
            return Ok(body);
        }
        //本地没有读取云端数据
        let result = self
            .read_content_from_cloud(file_block_meta.id.unwrap())
            .await;
        if let Err(e) = result {
            return Err(e);
        }
        let result = result?;
        //然后写入到本地
        let result = self.write_local_file(local_cache_file.clone(), result.as_slice(), 0);
        if let Ok(()) = result {
            return Err(ErrorInfo::Retry);
        }
        Err(result.err().unwrap())
    }
    fn write_local_file(&self, file_path: String, body: &[u8], seek: u64) -> ResponseResult<()> {
        let path = Path::new(file_path.as_str());
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                let parent_path = parent.to_str().unwrap();
                let result = fs::create_dir_all(parent_path);
                match result {
                    Ok(_) => {
                        info!("{}创建完成", parent_path)
                    }
                    Err(e) => {
                        error!("{}:{:?}创建失败", parent_path, e)
                    }
                }
            }
        }
        let result = File::options().create(true).write(true).open(file_path);
        if let Err(e) = result {
            let message = e.to_string();
            return Err(ErrorInfo::new_string(10, message));
        }
        let mut file = result?;
        let result = file.seek(Start(seek));
        if let Err(e) = result {
            return Err(ErrorInfo::new_string(12, e.to_string()));
        }
        debug!("跳过成功");
        let result = file.write(body);
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ErrorInfo::new_string(13, format!("文件写入失败:{}", e))),
        }
    }

    async fn read_content_from_cloud(&self, file_block_id: i32) -> ResponseResult<Vec<u8>> {
        // let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        info!("read from file_block_id:{}", file_block_id);
        let result = STORAGE_FACADE.write().await.content(file_block_id).await;

        let a = match result {
            Ok(result) => {
                let vec = result.to_vec();
                Ok(vec)
            }
            Err(e) => Err(e),
        };
        info!("read from :{} end", file_block_id);
        a
    }
    pub(crate) async fn write(&mut self, ino: u64, offset: u64, data: &[u8]) -> ResponseResult<u32> {
        let mut f = CONTEXT.file_manager.info_by_id(ino).await?;

        let block_index = (offset as usize) / CLOUD_FILE_BLOCK_SIZE;
        let block_offset = (offset as usize) % CLOUD_FILE_BLOCK_SIZE;

        let block_size = block_offset + data.len();
        let temp;
        if block_size > CLOUD_FILE_BLOCK_SIZE {
            let sub = data.len() - (block_size - CLOUD_FILE_BLOCK_SIZE);
            temp = &data[0..sub];
            self.upload_block_content(f.id.unwrap(), block_index as i64, block_offset as u64, temp)
                .await?;
            f.file_length = (offset as usize + temp.len()) as u64;
            Ok(temp.len() as u32)
        } else {
            f.file_length = (offset as usize + data.len()) as u64;
            temp = data;
            self.upload_block_content(f.id.unwrap(), block_index as i64, block_offset as u64, temp)
                .await?;
            CONTEXT.file_manager.update_meta(f).await?;
            Ok(data.len() as u32)
        }
    }
    async fn upload_block_content(&mut self, file_meta_id: u64, block_index: i64, seek: u64, data: &[u8]) -> ResponseResult<()> {
        let file_block_meta_opt = CONTEXT
            .file_manager
            .file_block_meta_index(file_meta_id, block_index)
            .await?;
        let mut md5 = Md5::new();
        md5.input(data);
        let md5_value = md5.result_str();
        let file_block_meta = match file_block_meta_opt {
            None => {
                let mut md5 = Md5::new();
                let file_name = format!("{}:{}", file_meta_id, block_index);
                let file_name_body = file_name.as_bytes();
                md5.input(file_name_body);
                let file_name_hash = md5.result_str();

                FileBlockMeta {
                    id: None,
                    file_part_id: file_name_hash,
                    block_index,
                    update_time: chrono::Local::now().timestamp_millis() as u64,
                    file_modify_time: chrono::Local::now().timestamp_millis() as u64,
                    file_meta_id,
                    deleted: 0,
                    part_hash: md5_value,
                    status: FileStatus::Init.into(),
                }
            }
            Some(mut f) => {
                f.part_hash = md5_value;
                f
            }
        };
        self.write_block(&file_block_meta, data, seek);
        CONTEXT
            .file_manager
            .save_file_block_meta(file_block_meta)
            .await?;
        Ok(())
    }
    fn write_block(&mut self, file_block_meta: &FileBlockMeta, body: &[u8], seek: u64) {
        let local_cache_file = format!("{}/{}", self.cache_file, file_block_meta.file_part_id);
        let result = self.write_local_file(local_cache_file, body, seek);
        match result {
            Ok(_) => {
                debug!("文件写入成功")
            }
            Err(e) => {
                error!("文件写入失败,{}", e)
            }
        }
    }
}
