use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};

use fuser::{Errno, FileAttr, FileType, Generation, INodeNo, ReplyEmpty, ReplyEntry};
use log::error;

use crate::fs::vfs::VirtualFileSystem;
use api::error::ErrorInfo;
use persistence::meta::FileMeta;
use persistence::FileMetaType;
use persistence::FileMetaType::FILE;

// 1 second
pub const TTL: Duration = Duration::new(1, 0);

pub struct CloudFuseFS {
    pub(crate) vfs: VirtualFileSystem,
}

unsafe impl Send for CloudFuseFS {}

impl CloudFuseFS {
    pub(crate) fn from(cache_file: &str) -> CloudFuseFS {
        CloudFuseFS {
            vfs: VirtualFileSystem::new(cache_file),
        }
    }
    pub(crate) fn convert_attr(&self, f: FileMeta) -> FileAttr {
        let kind;
        let perm=f.mode as u16;
        if f.file_type == FILE.get_code() {
            kind = FileType::RegularFile;
        } else {
            kind = FileType::Directory;
        }
        let uid = f.uid as u32;
        let gid = f.gid as u32;

        let create_time = UNIX_EPOCH + Duration::from_millis(f.create_time.timestamp_millis() as u64);

        let update_time = UNIX_EPOCH + Duration::from_millis(f.update_time.timestamp_millis() as u64);
        FileAttr {
            ino: INodeNo(f.id.unwrap() as u64),
            size: f.file_length as u64,
            blocks: 1,
            atime: update_time,
            mtime: update_time,
            ctime: create_time,
            crtime: create_time,
            kind,
            perm,
            nlink: 1,
            uid,
            gid,
            rdev: 0,
            blksize: 0,
            flags: 0,
        }
    }
    pub(crate) fn create_file(
        &self,
        parent: u64,
        name: &OsStr,
        reply: ReplyEntry,
        file_type: FileMetaType,
    ) {
        let Some(name_str) = name.to_str() else {
            reply.error(Errno::EINVAL);
            return;
        };
        crate::fs::runtime::block_on(async {
                let result = self.vfs.create_file(parent, name_str, file_type).await;
                if let Err(e) = result {
                    if let ErrorInfo::FileAlreadyExist(name) = e {
                        error!("文件已经存在:{}", name);
                        reply.error(Errno::EEXIST);
                        return;
                    }
                    error!("create_file:文件创建失败:{}", e);
                    reply.error(Errno::ENOSYS);
                    return;
                }
                let meta = result.unwrap();

                let attr = self.convert_attr(meta);
                let data = vec![];
                let data = data.as_slice();
                self.vfs.write(attr.ino.into(), 0, data).await.ok();
                reply.entry(&TTL, &attr, Generation(0));
            });
    }

    pub(crate) fn del_file(&self, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let Some(name_str) = name.to_str() else {
            reply.error(Errno::EINVAL);
            return;
        };
        let result = crate::fs::runtime::block_on(self.vfs.del_file(parent, name_str));
        if let Err(e) = result {
            if let ErrorInfo::FileNotFound(name) = e {
                error!("文件不存在:{}", name);
                reply.error(Errno::ENOENT);
                return;
            }
            error!("文件删除失败:{}", e);
            reply.error(Errno::ENOSYS);
            return;
        }
        reply.ok();
    }
}
