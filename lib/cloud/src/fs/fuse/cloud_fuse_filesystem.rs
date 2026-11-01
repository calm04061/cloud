use std::ffi::{OsStr, OsString};
use std::time::SystemTime;

use fuser::{
    BsdFileFlags, Errno, FileHandle, FileType, Filesystem, Generation, INodeNo, LockOwner,
    OpenFlags, RenameFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite,
    Request, TimeOrNow, WriteFlags,
};
use log::{debug, error, info, warn};

use api::error::ErrorInfo;
use persistence::FileMetaType::{DIR, FILE};

use crate::fs::fuse::cloud_fuse_fs::{CloudFuseFS, TTL};

impl Filesystem for CloudFuseFS {
    fn destroy(&mut self) {
        info!("正在 umount")
    }

    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        // debug!("lookup");
        let Some(name_str) = name.to_str() else {
            reply.error(Errno::EINVAL);
            return;
        };
        let result = crate::fs::runtime::block_on(self.vfs.lookup(parent.into(), name_str));
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.entry(&TTL, &attr, Generation(0));
            }
            Err(e) => {
                warn!("lookup:{}", e);
                reply.error(Errno::ENOENT);
            }
        }
    }
    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        // debug!("getattr");
        let result = crate::fs::runtime::block_on(self.vfs.file_info(ino.into()));
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.attr(&TTL, &attr)
            }
            Err(e) => {
                error!("getattr:{}", e);
                reply.error(Errno::ENOENT);
            }
        }
    }
    fn setattr(&self, _req: &Request, ino: INodeNo, mode: Option<u32>,
               uid: Option<u32>, gid: Option<u32>, size: Option<u64>, _atime: Option<TimeOrNow>,
               mtime: Option<TimeOrNow>, _ctime: Option<SystemTime>, _fh: Option<FileHandle>, _crtime: Option<SystemTime>,
               _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, _flags: Option<BsdFileFlags>,
               reply: ReplyAttr,
    ) {
        let mtime_millis = mtime.map(|t| match t {
            TimeOrNow::SpecificTime(time) => time
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            TimeOrNow::Now => chrono::Local::now().timestamp_millis(),
        });
        let result = crate::fs::runtime::block_on(self.vfs.setattr(
            ino.into(),
            size,
            mode,
            uid,
            gid,
            mtime_millis,
        ));
        match result {
            Ok(f) => {
                let attr = self.convert_attr(f);
                reply.attr(&TTL, &attr)
            }
            Err(e) => {
                error!("setattr:{}", e);
                reply.error(Errno::ENOENT);
            }
        }
    }
    /// 创建文件
    fn mknod(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        // debug!("mknod");
        self.create_file(parent.into(), name, reply, FILE);
    }
    fn mkdir(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        // debug!("mkdir");
        self.create_file(parent.into(), name, reply, DIR);
    }
    fn unlink(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        info!("unlink");
        self.del_file(parent.into(), name, reply);
    }
    fn rmdir(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEmpty) {
        info!("rmdir");
        self.del_file(parent.into(), name, reply);
    }

    fn rename(
        &self,
        _req: &Request,
        parent: INodeNo,
        name: &OsStr,
        new_parent: INodeNo,
        new_name: &OsStr,
        _flags: RenameFlags,
        reply: ReplyEmpty,
    ) {
        let (Some(name_str), Some(new_name_str)) = (name.to_str(), new_name.to_str()) else {
            reply.error(Errno::EINVAL);
            return;
        };
        crate::fs::runtime::block_on(async {
                info!("rename");
                let result = self.vfs.rename(
                    parent.into(),
                    name_str,
                    new_parent.into(),
                    new_name_str,
                ).await;
                if let Err(e) = result {
                    let out = match e {
                        ErrorInfo::FileAlreadyExist(_) => Errno::EEXIST,
                        _ => {
                            error!("rename:{}", e);
                            Errno::ENOENT
                        }
                    };
                    reply.error(out);
                    return;
                }
                reply.ok();
            });
    }
    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        crate::fs::runtime::block_on(async {
                debug!("read file id {},from {:?}:size={}", ino, offset, size);
                let result = self.vfs.read(ino.into(), offset, size).await;
                if let Err(e) = result {
                    error!("read:{}", e);
                    reply.error(Errno::ENOENT);
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
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        data: &[u8],
        _write_flags: WriteFlags,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyWrite,
    ) {
        crate::fs::runtime::block_on(async {
                let result = self.vfs.write(ino.into(), offset, data).await;
                match result {
                    Ok(len) => reply.written(len),
                    Err(e) => {
                        error!("write:{}", e);
                        reply.error(Errno::ENOSYS);
                        return;
                    }
                }
            })
    }
    // fn flush(&mut self, _req: &Request<'_>, ino: u64, fh: u64, lock_owner: u64, reply: ReplyEmpty) {
    //     todo!()
    // }
    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        crate::fs::runtime::block_on(async {
                let list = self.vfs.list_by_parent(ino.into()).await;
                let children = match list {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}", e);
                        reply.ok();
                        return;
                    }
                };
                // 位置 0/1 保留给 "." 和 ".."，其后为子项；offset 为客户端上次消费到的位置
                let mut entries: Vec<(INodeNo, FileType, OsString)> = Vec::with_capacity(children.len() + 2);
                entries.push((ino, FileType::Directory, OsString::from(".")));
                entries.push((ino, FileType::Directory, OsString::from("..")));
                for file in children {
                    let kind;
                    if file.file_type == FILE.get_code() {
                        kind = FileType::RegularFile;
                    } else {
                        kind = FileType::Directory;
                    }
                    let Some(child_ino) = file.id else {
                        continue;
                    };
                    entries.push((INodeNo(child_ino as u64), kind, OsString::from(file.name)));
                }
                for (index, (entry_ino, kind, name)) in entries.iter().enumerate().skip(offset as usize) {
                    info!("readdir:{}:{}:{}", ino, index, name.to_string_lossy());
                    // add 返回 true 表示客户端缓冲区已满，客户端会携带下一次 offset 继续调用
                    if reply.add(*entry_ino, (index as u64) + 1, *kind, name) {
                        break;
                    }
                }
                reply.ok();
            })
        // info!("readdir:{}:{:?}",ino,req);
    }
}
