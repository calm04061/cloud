use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};

use fuser::{FileAttr, FileType, ReplyAttr, ReplyEmpty, ReplyEntry};
use libc::{EEXIST, ENOENT, ENOSYS};
use log::error;
use users::{get_current_gid, get_current_uid};
use persistence::{FileMeta, FileMetaType};
use persistence::FileMetaType::FILE;

use crate::error::ErrorInfo;
use crate::fs::vfs::VirtualFileSystem;

// 1 second
pub const TTL: Duration = Duration::new(1, 0);

pub struct CloudFS {
    pub(crate) vfs: VirtualFileSystem,
}

unsafe impl Send for CloudFS {}

impl CloudFS {
    pub(crate) fn from(cache_file: &str) -> CloudFS {
        CloudFS {
            vfs: VirtualFileSystem::new(cache_file),
        }
    }
    pub(crate) fn convert_attr(&self, f: FileMeta) -> FileAttr {
        let kind;
        let perm;
        if f.file_type == FILE.get_code() {
            kind = FileType::RegularFile;
            perm = 0o644;
        } else {
            kind = FileType::Directory;
            perm = 0o755;
        }
        let uid = get_current_uid();
        let gid = get_current_gid();

        let create_time = UNIX_EPOCH + Duration::from_millis(f.create_time as u64);

        let update_time = UNIX_EPOCH + Duration::from_millis(f.update_time as u64);
        FileAttr {
            ino: f.id.unwrap() as u64,
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
        &mut self,
        parent: u64,
        name: &OsStr,
        reply: ReplyEntry,
        file_type: FileMetaType,
    ) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let name_str = name.to_str().unwrap();
                let result = self.vfs.create_file(parent, name_str, file_type).await;
                if let Err(e) = result {
                    if let ErrorInfo::FileAlreadyExist(name) = e {
                        error!("文件已经存在:{}", name);
                        reply.error(EEXIST);
                        return;
                    }
                    error!("create_file:文件创建失败:{}", e);
                    reply.error(ENOSYS);
                    return;
                }
                let meta = result.unwrap();

                let attr = self.convert_attr(meta);
                let data = vec![];
                let data = data.as_slice();
                self.vfs.write(attr.ino, 0, data).await.ok();
                reply.entry(&TTL, &attr, 0);
            });
    }

    pub(crate) fn del_file(&self, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let result = self.vfs.del_file(parent, name.to_str().unwrap());
        if let Err(e) = result {
            if let ErrorInfo::FileNotFound(name) = e {
                error!("文件不存在:{}", name);
                reply.error(ENOENT);
                return;
            }
            error!("文件删除失败:{}", e);
            reply.error(ENOSYS);
            return;
        }
        reply.ok();
    }

    pub(crate) fn file_attr(&self, option: Option<FileMeta>, reply: ReplyAttr) {
        match option {
            None => reply.error(ENOENT),
            Some(f) => {
                let attr = self.convert_attr(f);
                reply.attr(&TTL, &attr)
            }
        }
    }
}
