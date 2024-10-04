use crate::fs::vfs::VirtualFileSystem;
use nfsserve::nfs::{fattr3, fileid3, filename3, ftype3, mode3, nfsstring, nfstime3, size3, specdata3};
use persistence::meta::FileMeta;
use persistence::FileMetaType;
use tokio::sync::RwLock;

pub struct CloudNFS {
    pub(crate) vfs: RwLock<VirtualFileSystem>,
}

impl CloudNFS {
    pub fn new(cache_file: &str) -> CloudNFS {
        CloudNFS {
            vfs: RwLock::new(VirtualFileSystem::new(cache_file)),
        }
    }
    pub(crate) fn convert_name2string(filename: &filename3) -> String {
        let name = filename.to_vec();
        let name = name.as_slice();
        let name = String::from_utf8_lossy(name);
        let name = name.to_string();
        name
    }
    pub(crate) fn convert_name2filename(name: &str) -> nfsstring {
        let bytes: &[u8] = name.as_bytes();
        let slice = filename3::from(bytes);
        slice
    }
    pub(crate) fn convert_fattr3(meta: &FileMeta) -> fattr3 {
        let mut fattr = fattr3::default();
        let file_type: FileMetaType = meta.file_type.into();
        fattr.ftype = match file_type {
            FileMetaType::FILE => {
                ftype3::NF3REG
            }
            FileMetaType::DIR => {
                ftype3::NF3DIR
            }
            FileMetaType::SYMLINK => {
                ftype3::NF3LNK
            }
        };
        fattr.mode = meta.mode as mode3;
        fattr.nlink = 1;
        fattr.gid = 1000;
        fattr.uid = 1000;
        fattr.size = meta.file_length as size3;
        fattr.used = 0;
        fattr.rdev = specdata3::default();
        fattr.fsid = 0;
        fattr.fileid = meta.id.unwrap() as fileid3;

        fattr.ctime = nfstime3 {
            seconds: (meta.create_time.timestamp_millis()/1000) as u32,
            nseconds: 0,
        };
        fattr.mtime = nfstime3 {
            seconds: (meta.update_time.timestamp_millis()/1000) as u32,
            nseconds: 0,
        };
        fattr.atime = nfstime3 {
            seconds: (meta.update_time.timestamp_millis()/1000) as u32,
            nseconds: 0,
        };
        fattr
    }
}
