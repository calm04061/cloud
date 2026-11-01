use log::{debug, error, info};
use std::fs;
use std::fs::File;
use std::io::SeekFrom::Start;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::sync::atomic::Ordering;

use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::meta::FileBlockMeta;
use storage::storage_facade;

use super::super::vfs::{CACHE_HITS, CACHE_MISSES};
use super::Inner;

/// 本地缓存职责：缓存文件读写与云端数据回填。
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
    pub(crate) async fn read_block(
        &self,
        file_block_meta: &FileBlockMeta,
        seek: u64,
    ) -> ResponseResult<Vec<u8>> {
        let local_cache_file = format!("{}/{}", self.cache_file, file_block_meta.file_part_id);
        // 本地已尽存在直接返回
        let result = self.read_local_file(local_cache_file.clone(), seek);
        if let Ok(body) = result {
            CACHE_HITS.fetch_add(1, Ordering::SeqCst);
            return Ok(body);
        }
        CACHE_MISSES.fetch_add(1, Ordering::SeqCst);
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
    pub(crate) fn write_local_file(&self, file_path: String, body: &[u8], seek: u64) -> ResponseResult<()> {
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
        info!("read from file_block_id:{}", file_block_id);
        let result = storage_facade().content(file_block_id).await;

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
}
