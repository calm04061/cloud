use std::fmt::Debug;
use std::io::{Error, ErrorKind, SeekFrom};
use std::io::SeekFrom::Start;
use std::time::SystemTime;

use bytes::{Buf, BufMut, Bytes};
use chrono::{Local, TimeZone};
use dav_server::fs::{DavDirEntry, DavFile, DavMetaData, FsFuture, FsResult};
use futures_util::{future, FutureExt};
use log::info;

use crate::database::meta::FileMetaType;
use crate::domain::table::tables::FileMeta;
use crate::fs::vfs::VirtualFileSystem;

const BLOCK_SIZE: usize = 1024 * 1024 * 4;

#[derive(Debug, Clone)]
pub struct CloudFsMetaData {
    pub(crate) file_meta: FileMeta,
}

impl CloudFsMetaData {
    pub(crate) fn new(file_meta: FileMeta) -> CloudFsMetaData {
        CloudFsMetaData {
            file_meta
        }
    }
}

impl DavMetaData for CloudFsMetaData {
    fn len(&self) -> u64 {
        self.file_meta.file_length as u64
    }

    fn modified(&self) -> FsResult<SystemTime> {
        let date = Local.timestamp_millis_opt(self.file_meta.create_time);
        let time = date.unwrap();
        let time = SystemTime::from(time);
        Ok(time)
    }

    fn is_dir(&self) -> bool {
        let file_type: FileMetaType = self.file_meta.file_type.into();
        return file_type == FileMetaType::DIR;
    }
}

#[derive(Debug, Clone)]
pub struct CloudDavDirEntry {
    pub(crate) file_meta: FileMeta,
}

impl CloudDavDirEntry {
    pub(crate) fn new(file_meta: FileMeta) -> CloudDavDirEntry {
        CloudDavDirEntry {
            file_meta
        }
    }
}

impl DavDirEntry for CloudDavDirEntry {
    fn name(&self) -> Vec<u8> {
        self.file_meta.name.clone().into_bytes()
    }

    fn metadata(&self) -> FsFuture<Box<dyn DavMetaData>> {
        Box::pin(future::ok(Box::new(CloudFsMetaData { file_meta: self.file_meta.clone() }) as Box<dyn DavMetaData>))
    }
}

#[derive(Debug)]
pub(crate) struct CloudDavFile {
    pub(crate) file_meta: FileMeta,
    pub(crate) fs: VirtualFileSystem,
    pos: usize,
    temp_pos: usize,
    temp_buf: Vec<u8>,
    // temp_file: Option<String>,
}

impl CloudDavFile {
    pub(crate) fn new(file_meta: FileMeta, system: VirtualFileSystem) -> CloudDavFile {
        CloudDavFile {
            file_meta,
            fs: system,
            pos: 0,
            temp_pos: 0,
            temp_buf: vec![],
        }
    }
}

impl DavFile for CloudDavFile {
    fn metadata(&mut self) -> FsFuture<Box<dyn DavMetaData>> {
        async move {
            let data = CloudFsMetaData::new(self.file_meta.clone());
            Ok(Box::new(data) as Box<dyn DavMetaData>)
        }.boxed()
    }

    fn write_buf(&mut self, _buf: Box<dyn Buf + Send>) -> FsFuture<()> {
        info!("write_buf,{},pos:{}", self.file_meta.name,self.pos);
        todo!()
    }

    fn write_bytes(&mut self, buf: Bytes) -> FsFuture<()> {
        async move {
            let id = self.file_meta.id.unwrap();

            self.temp_buf.put_slice(buf.as_ref());
            let temp_size = self.temp_buf.len();
            if temp_size > BLOCK_SIZE {
                info!("write_block_bytes,{}:{},pos:{},len:{}", id,self.file_meta.name,self.temp_pos,self.temp_buf.len());
                self.fs.write(id as u64, self.temp_pos as i64, self.temp_buf.as_ref()).await.unwrap();
                self.temp_buf.clear();
                self.pos += buf.len();
                self.temp_pos = self.pos;
            } else {
                let pre_pos = self.pos;
                self.pos += buf.len();
                info!("write_bytes,{}:{},pos:{} + len:{} = {}", id, self.file_meta.name, pre_pos, buf.len(), self.pos);
            }
            Ok(())
        }.boxed()
    }

    fn read_bytes(&mut self, count: usize) -> FsFuture<Bytes> {
        async move {
            let id = self.file_meta.id.unwrap();
            info!("read_bytes,{}:{},pos:{},count:{}", id ,self.file_meta.name ,self.pos,count);
            let result = self.fs.read(id as u64, self.pos as i64, count as u32).await.unwrap();
            let bytes = Bytes::copy_from_slice(result.as_slice());
            self.pos += count;
            Ok(bytes)
        }.boxed()
    }

    fn seek(&mut self, pos: SeekFrom) -> FsFuture<u64> {
        async move {
            let (start, offset): (u64, i64) = match pos {
                Start(npos) => {
                    self.pos = npos as usize;
                    return Ok(npos);
                }
                SeekFrom::Current(npos) => (self.pos as u64, npos),
                SeekFrom::End(npos) => {
                    let curlen = self.file_meta.file_length as u64;
                    (curlen, npos)
                }
            };
            if offset < 0 {
                if -offset as u64 > start {
                    return Err(Error::new(ErrorKind::InvalidInput, "invalid seek").into());
                }
                self.pos = (start - (-offset as u64)) as usize;
            } else {
                self.pos = (start + offset as u64) as usize;
            }
            Ok(self.pos as u64)
        }
            .boxed()
    }

    fn flush(&mut self) -> FsFuture<()> {
        async move {
            info!("{}:flush",self.file_meta.id.unwrap());
            let id = self.file_meta.id.unwrap();
            let temp_size = self.temp_buf.len();
            if temp_size > 0 {
                self.fs.write(id as u64, self.temp_pos as i64, self.temp_buf.as_ref()).await.unwrap();
                self.temp_buf.clear();
            }
            self.pos = 0;
            self.temp_pos = 0;
            Ok(())
        }.boxed()
    }
}
