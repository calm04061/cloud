use crypto::digest::Digest;
use crypto::md5::Md5;
use log::{debug, error};
use std::sync::atomic::Ordering;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::meta::FileBlockMeta;
use persistence::FileStatus;
use service::meta::FileManager;
use service::CONTEXT;

use super::{file_lock, VirtualFileSystem, CLOUD_FILE_BLOCK_SIZE, IN_FLIGHT_WRITES};

/// 块编排职责：VFS 读写入口，按块拆分/聚合并委托 cache 与 file_manager。
impl VirtualFileSystem {
    ///
    /// 循环写入多个文件块
    ///
    pub(crate) async fn write(&self, ino: u64, offset: u64, data: &[u8]) -> ResponseResult<u32> {
        let _write_lock = file_lock(ino as i64).await;
        IN_FLIGHT_WRITES.fetch_add(1, Ordering::SeqCst);
        let result = self.write_inner(ino, offset, data).await;
        IN_FLIGHT_WRITES.fetch_sub(1, Ordering::SeqCst);
        result
    }

    async fn write_inner(&self, ino: u64, offset: u64, data: &[u8]) -> ResponseResult<u32> {
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

    pub(crate) async fn read(&self, ino: u64, offset: u64, size: u32) -> ResponseResult<Vec<u8>> {
        debug!("read file id {},from {:?}:size={}", ino, offset, size);
        let result = CONTEXT.file_manager.info_by_id(ino as i64).await;
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
}

/// Inner 的块写入细节：拆块、hash、元数据落库。
impl super::Inner {
    pub(crate) async fn write(&self, ino: u64, offset: u64, data: &[u8]) -> ResponseResult<u32> {
        let mut f = CONTEXT.file_manager.info_by_id(ino as i64).await?;

        let block_index = (offset as usize) / CLOUD_FILE_BLOCK_SIZE;
        let block_offset = (offset as usize) % CLOUD_FILE_BLOCK_SIZE;

        let block_size = block_offset + data.len();
        let temp;
        if block_size > CLOUD_FILE_BLOCK_SIZE {
            let sub = data.len() - (block_size - CLOUD_FILE_BLOCK_SIZE);
            temp = &data[0..sub];
            self.upload_block_content(f.id.unwrap(), block_index as i64, block_offset as u64, temp)
                .await?;
            let file_length = (offset as usize + temp.len()) as i64;
            f.file_length = file_length;
            CONTEXT.file_manager.update_meta(f).await?;
            Ok(temp.len() as u32)
        } else {
            f.file_length = (offset as usize + data.len())  as i64;
            temp = data;
            self.upload_block_content(f.id.unwrap(), block_index as i64, block_offset as u64, temp)
                .await?;
            CONTEXT.file_manager.update_meta(f).await?;
            Ok(data.len() as u32)
        }
    }
    async fn upload_block_content(&self, file_meta_id: i64, block_index: i64, seek: u64, data: &[u8]) -> ResponseResult<()> {
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
                    update_time: chrono::Local::now(),
                    file_modify_time: chrono::Local::now().timestamp_millis(),
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
    fn write_block(&self, file_block_meta: &FileBlockMeta, body: &[u8], seek: u64) {
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
