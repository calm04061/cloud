use std::ffi::OsStr;
use std::time::SystemTime;

use fuser::{
    FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite,
    Request, TimeOrNow,
};
use libc::{EEXIST, ENOENT, ENOSYS};
use log::{debug, error, info, warn};

use api::error::ErrorInfo;
use persistence::FileMetaType::{DIR, FILE};

use crate::fs::fuse::cloud_fuse_fs::{CloudFuseFS, TTL};

impl Filesystem for CloudFuseFS {
    fn destroy(&mut self) {
        info!("正在 umount")
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // debug!("lookup");
        let result = self.vfs.lookup(parent, name.to_str().unwrap());
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.entry(&TTL, &attr, 0);
            }
            Err(e) => {
                warn!("lookup:{}", e);
                reply.error(ENOENT);
            }
        }
    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        // debug!("getattr");
        let result = self.vfs.file_info(ino);
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.attr(&TTL, &attr)
            }
            Err(e) => {
                error!("getattr:{}", e);
                reply.error(ENOENT);
            }
        }
    }
    fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>,
               _uid: Option<u32>, _gid: Option<u32>, size: Option<u64>, _atime: Option<TimeOrNow>,
               _mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>, _fh: Option<u64>, _crtime: Option<SystemTime>,
               _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, _flags: Option<u32>,
               reply: ReplyAttr,
    ) {
        let result = self.vfs.setattr(ino, size);
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.attr(&TTL, &attr)
            }
            Err(e) => {
                error!("setattr:{}", e);
                reply.error(ENOENT);
            }
        }
    }
    /// 创建文件
    fn mknod(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        // debug!("mknod");
        self.create_file(parent, name, reply, FILE);
    }
    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        // debug!("mkdir");
        self.create_file(parent, name, reply, DIR);
    }
    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        info!("unlink");
        self.del_file(parent, name, reply);
    }
    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        info!("rmdir");
        self.del_file(parent, name, reply);
    }

    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        new_parent: u64,
        new_name: &OsStr,
        _flags: u32,
        reply: ReplyEmpty,
    ) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                info!("rename");
                let result = self.vfs.rename(
                    parent,
                    name.to_str().unwrap(),
                    new_parent,
                    new_name.to_str().unwrap(),
                ).await;
                if let Err(e) = result {
                    let out = match e {
                        ErrorInfo::FileAlreadyExist(_) => EEXIST,
                        _ => {
                            error!("rename:{}", e);
                            ENOENT
                        }
                    };
                    reply.error(out);
                    return;
                }
                reply.ok();
            });
    }
    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                debug!("read file id {},from {:?}:size={}", ino, offset, size);
                let result = self.vfs.read(ino, offset as u64, size).await;
                if let Err(e) = result {
                    error!("read:{}", e);
                    reply.error(ENOENT);
                    return;
                }
                let body = result.unwrap();
                info!(
                    "read file id {},from {:?}:size={},real size={}",
                    ino,
                    offset,
                    size,
                    body.len()
                );
                reply.data(body.as_slice());
            });
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = self.vfs.write(ino, offset as u64, data).await;
                match result {
                    Ok(len) => reply.written(len),
                    Err(e) => {
                        error!("write:{}", e);
                        reply.error(ENOSYS);
                        return;
                    }
                }
            })
    }
    // fn flush(&mut self, _req: &Request<'_>, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
    //     todo!()
    // }
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                if offset == 0 {
                    let _ = reply.add(ino, 0, FileType::Directory, ".");
                    let _ = reply.add(ino, 1, FileType::Directory, "..");
                    let vec = self.vfs.list_by_parent(ino).await;
                    if let Err(e) = vec {
                        error!("{}", e);
                        reply.ok();
                        return;
                    }
                    let vec = vec.unwrap();
                    let mut index = 2;
                    for file in vec {
                        let kind;
                        if file.file_type == FILE.get_code() {
                            kind = FileType::RegularFile;
                        } else {
                            kind = FileType::Directory;
                        }
                        info!("readdir:{}:{}", ino, file.name);
                        let _ = reply.add(file.id.unwrap() as u64, index, kind, file.name);
                        index = index + 1;
                    }
                }
                reply.ok();
            })
        // info!("readdir:{}:{:?}",ino,req);
    }
}
