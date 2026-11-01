use crate::fs::nfs::cloud_nfs::CloudNFS;
use api::ROOT_FILE_ID;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use log::{error, info};
use nfsserve::nfs::set_atime::SET_TO_CLIENT_TIME;
use nfsserve::nfs::set_gid3::gid;
use nfsserve::nfs::set_mode3::mode;
use nfsserve::nfs::set_size3::size;
use nfsserve::nfs::set_uid3::uid;
use nfsserve::nfs::{fattr3, fileid3, filename3, nfspath3, nfsstat3, sattr3, set_atime};
use nfsserve::vfs::{DirEntry, NFSFileSystem, ReadDirResult, VFSCapabilities};
use persistence::FileMetaType;

#[async_trait]
impl NFSFileSystem for CloudNFS {
    fn capabilities(&self) -> VFSCapabilities {
        VFSCapabilities::ReadWrite
    }

    fn root_dir(&self) -> fileid3 {
        ROOT_FILE_ID
    }

    async fn lookup(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        // info!("lookup: {} ", dirid);
        let name = Self::convert_name2string(filename);
        let name = name.as_str();
        let vfs = &self.vfs;

        let result = vfs.lookup(dirid, name).await;
        if let Err(x) = result {
            error!("lookup: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let result = result.unwrap();
        let Some(id) = result.id else {
            return Err(nfsstat3::NFS3ERR_IO);
        };
        Ok(id as fileid3)
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        // info!("getattr: {}", id);
        let vfs = &self.vfs;
        let meta = vfs.file_info(id as u64).await.map_err(|x| {
            error!("getattr: {}", x);
            nfsstat3::NFS3ERR_NOENT
        })?;
        Ok(Self::convert_fattr3(&meta))
    }

    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        // info!("setattr: {}", id);
        let vfs = &self.vfs;
        let mut meta = vfs.file_info(id as u64).await.map_err(|x| {
            error!("setattr: {}", x);
            nfsstat3::NFS3ERR_NOENT
        })?;
        if let mode(mode_value) = setattr.mode {
            meta.mode = mode_value as i32;
        }
        if let uid(uid_value) = setattr.uid {
            meta.uid = uid_value as i32;
        }
        if let gid(gid_value) = setattr.gid {
            meta.gid = gid_value as i32;
        }
        if let size(size_value) = setattr.size {
            meta.file_length = size_value as i64;
        }
        match setattr.atime {
            set_atime::DONT_CHANGE => {}
            set_atime::SET_TO_SERVER_TIME => {
                meta.update_time = Local::now();
            }
            SET_TO_CLIENT_TIME(time) => {
                match DateTime::from_timestamp(time.seconds as i64, time.nseconds) {
                    Some(time) => meta.update_time = time.into(),
                    None => return Err(nfsstat3::NFS3ERR_INVAL),
                }
            }
        }
        let meta = vfs.update_file_meta(meta).await.map_err(|x| {
            error!("setattr: {}", x);
            nfsstat3::NFS3ERR_IO
        })?;
        Ok(Self::convert_fattr3(&meta))
    }

    async fn read(&self, id: fileid3, offset: u64, count: u32) -> Result<(Vec<u8>, bool), nfsstat3> {
        // info!("read: {}", id);
        let vfs = &self.vfs;
        let meta = vfs.file_info(id).await;
        if let Err(x) = meta {
            error!("read: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let meta = meta.unwrap();
        let vec = vfs.read(id, offset, count).await;
        if let Err(x) = vec {
            error!("read: {}", x);
            return Err(nfsstat3::NFS3ERR_IO);
        }
        let vec = vec.unwrap();
        let end = meta.file_length <= (offset + (count as u64)) as i64;
        Ok((vec, end))
    }

    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        // info!("write: {}", id);
        let vfs = &self.vfs;
        vfs.write(id, offset, data).await.map_err(|x| {
            error!("write: {}", x);
            nfsstat3::NFS3ERR_IO
        })?;
        let meta = vfs.file_info(id).await.map_err(|x| {
            error!("write: {}", x);
            nfsstat3::NFS3ERR_NOENT
        })?;
        Ok(Self::convert_fattr3(&meta))
    }

    async fn create(&self, dirid: fileid3, filename: &filename3, _attr: sattr3) -> Result<(fileid3, fattr3), nfsstat3> {
        // info!("create: {}", dirid);
        let vfs = &self.vfs;
        let parent_file = vfs.file_info(dirid as u64).await.map_err(|x| {
            error!("create: {}", x);
            nfsstat3::NFS3ERR_NOENT
        })?;
        let name = Self::convert_name2string(&filename);
        let meta = vfs.create_file(parent_file.id.unwrap_or(0) as u64, &name, FileMetaType::FILE).await
            .map_err(|x| {
                error!("create: {}", x);
                if matches!(x, api::error::ErrorInfo::FileAlreadyExist(_)) {
                    return nfsstat3::NFS3ERR_EXIST;
                }
                nfsstat3::NFS3ERR_IO
            })?;
        let Some(id) = meta.id else {
            return Err(nfsstat3::NFS3ERR_IO);
        };
        Ok((id as fileid3, Self::convert_fattr3(&meta)))
    }

    async fn create_exclusive(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        info!("create_exclusive: {}", dirid);
        let name = Self::convert_name2string(filename);
        let name = name.as_str();
        let vfs = &self.vfs;
        if vfs.lookup(dirid, name).await.is_ok() {
            return Err(nfsstat3::NFS3ERR_EXIST);
        }
        let meta = vfs.create_file(dirid, name, FileMetaType::FILE).await.map_err(|x| {
            error!("create_exclusive: {}", x);
            if matches!(x, api::error::ErrorInfo::FileAlreadyExist(_)) {
                return nfsstat3::NFS3ERR_EXIST;
            }
            nfsstat3::NFS3ERR_IO
        })?;
        let Some(id) = meta.id else {
            return Err(nfsstat3::NFS3ERR_IO);
        };
        Ok(id as fileid3)
    }

    async fn mkdir(&self, dirid: fileid3, dirname: &filename3) -> Result<(fileid3, fattr3), nfsstat3> {
        // info!("mkdir: {}", dirid);
        let vfs = &self.vfs;
        let name = Self::convert_name2string(dirname);
        let name = name.as_str();
        let meta = vfs.create_file(dirid, name, FileMetaType::DIR).await.map_err(|x| {
            error!("mkdir: {}", x);
            if matches!(x, api::error::ErrorInfo::FileAlreadyExist(_)) {
                return nfsstat3::NFS3ERR_EXIST;
            }
            nfsstat3::NFS3ERR_IO
        })?;
        let Some(id) = meta.id else {
            return Err(nfsstat3::NFS3ERR_IO);
        };
        let fattr = Self::convert_fattr3(&meta);
        Ok((id as fileid3, fattr))
    }

    async fn remove(&self, dirid: fileid3, filename: &filename3) -> Result<(), nfsstat3> {
        // info!("remove: {}", dirid);
        let vfs = &self.vfs;
        let name = Self::convert_name2string(filename);
        let name = name.as_str();
        vfs.del_file(dirid, name).await.map_err(|x| {
            error!("remove: {}", x);
            if matches!(x, api::error::ErrorInfo::FileNotFound(_)) {
                return nfsstat3::NFS3ERR_NOENT;
            }
            nfsstat3::NFS3ERR_IO
        })?;
        Ok(())
    }

    async fn rename(&self, from_dirid: fileid3, from_filename: &filename3, to_dirid: fileid3, to_filename: &filename3) -> Result<(), nfsstat3> {
        // info!("rename: {} {}", from_dirid,  to_dirid);
        let vfs = &self.vfs;
        let from_name = Self::convert_name2string(from_filename);
        let from_name = from_name.as_str();
        let to_filename = Self::convert_name2string(to_filename);
        let to_filename = to_filename.as_str();
        vfs.rename(from_dirid, from_name, to_dirid, to_filename).await.map_err(|x| {
            error!("rename: {}", x);
            match x {
                api::error::ErrorInfo::FileNotFound(_) => nfsstat3::NFS3ERR_NOENT,
                api::error::ErrorInfo::FileAlreadyExist(_) => nfsstat3::NFS3ERR_EXIST,
                _ => nfsstat3::NFS3ERR_IO,
            }
        })?;
        Ok(())
    }

    async fn readdir(&self, dirid: fileid3, start_after: fileid3, max_entries: usize) -> Result<ReadDirResult, nfsstat3> {
        // info!("readdir: {}", dirid);
        let vfs = &self.vfs;
        let result = vfs.list_by_parent_page(dirid, start_after, max_entries).await;
        if let Err(x) = result {
            error!("readdir: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let (list, end) = result.unwrap();

        let mut vec = Vec::new();
        for item in list {
            let fattr3 = Self::convert_fattr3(&item);
            let filename = Self::convert_name2filename(&item.name);
            let Some(id) = item.id else {
                return Err(nfsstat3::NFS3ERR_IO);
            };
            let entry = DirEntry {
                fileid: id as fileid3,
                name: filename,
                attr: fattr3,
            };
            vec.push(entry);
        }
        Ok(ReadDirResult {
            entries: vec,
            end,
        })
    }

    async fn symlink(&self, dirid: fileid3, linkname: &filename3, symlink: &nfspath3, _attr: &sattr3) -> Result<(fileid3, fattr3), nfsstat3> {
        info!("symlink: {} {:?} -> {:?}", dirid, linkname, symlink);
        // 元数据模型暂无 symlink 目标存储列，先显式返回不支持
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }

    async fn readlink(&self, id: fileid3) -> Result<nfspath3, nfsstat3> {
        info!("readlink: {}", id);
        Err(nfsstat3::NFS3ERR_NOTSUPP)
    }
}
