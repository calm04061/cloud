use crate::fs::nfs::cloud_nfs::CloudNFS;
use api::ROOT_FILE_ID;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
        // let root_id = tokio::runtime::Builder::new_multi_thread()
        //     .enable_all()
        //     .build()
        //     .unwrap()
        //     .block_on(async {
        //         let guard = self.vfs.read().await;
        //         let option = guard.path_meta("/").await.unwrap();
        //         option.unwrap().id.unwrap() as fileid3
        //     });
        // root_id
        ROOT_FILE_ID
    }

    async fn lookup(&self, dirid: fileid3, filename: &filename3) -> Result<fileid3, nfsstat3> {
        // info!("lookup: {} ", dirid);
        let name = Self::convert_name2string(filename);
        let name = name.as_str();
        let guard = self.vfs.write().await;

        let result = guard.lookup_sync(dirid, name).await;
        if let Err(x) = result {
            error!("lookup: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let result = result.unwrap();
        Ok(result.id.unwrap() as fileid3)
    }

    async fn getattr(&self, id: fileid3) -> Result<fattr3, nfsstat3> {
        // info!("getattr: {}", id);
        let guard = self.vfs.read().await;
        let meta = guard.file_info_sync(id as u64).await.unwrap();
        Ok(Self::convert_fattr3(&meta))
    }

    async fn setattr(&self, id: fileid3, setattr: sattr3) -> Result<fattr3, nfsstat3> {
        // info!("setattr: {}", id);
        let guard = self.vfs.write().await;
        let mut meta = guard.file_info_sync(id as u64).await.unwrap();
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
                meta.update_time = Utc::now();
            }
            SET_TO_CLIENT_TIME(time) => {
                let option = DateTime::from_timestamp(time.seconds as i64, time.nseconds).unwrap();
                meta.update_time = option;
            }
        }
        let meta = guard.update_file_meta(meta).await.unwrap();
        Ok(Self::convert_fattr3(&meta))
    }

    async fn read(&self, id: fileid3, offset: u64, count: u32) -> Result<(Vec<u8>, bool), nfsstat3> {
        // info!("read: {}", id);
        let guard = self.vfs.read().await;
        let meta = guard.file_info_sync(id).await;
        if let Err(x) = meta {
            error!("read: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let meta = meta.unwrap();
        let vec = guard.read(id, offset, count).await;
        if let Err(x) = vec {
            error!("read: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let vec = vec.unwrap();
        let end = meta.file_length <= (offset + (count as u64)) as i64;
        Ok((vec, end))
    }

    async fn write(&self, id: fileid3, offset: u64, data: &[u8]) -> Result<fattr3, nfsstat3> {
        // info!("write: {}", id);
        let mut guard = self.vfs.write().await;
        guard.write(id, offset, data).await.unwrap();
        let meta = guard.file_info_sync(id).await.unwrap();
        Ok(Self::convert_fattr3(&meta))
    }

    async fn create(&self, dirid: fileid3, filename: &filename3, _attr: sattr3) -> Result<(fileid3, fattr3), nfsstat3> {
        // info!("create: {}", dirid);
        let guard = self.vfs.write().await;
        let parent_file = guard.file_info_sync(dirid as u64).await.unwrap();
        let name = Self::convert_name2string(&filename);
        let meta = guard.create_file(parent_file.id.unwrap() as u64, &name, FileMetaType::FILE).await.unwrap();
        Ok((meta.id.unwrap() as fileid3, Self::convert_fattr3(&meta)))
    }

    async fn create_exclusive(&self, _dirid: fileid3, _filename: &filename3) -> Result<fileid3, nfsstat3> {
        info!("create_exclusive: {}", _dirid);
        todo!()
    }

    async fn mkdir(&self, dirid: fileid3, dirname: &filename3) -> Result<(fileid3, fattr3), nfsstat3> {
        // info!("mkdir: {}", dirid);
        let guard = self.vfs.write().await;
        let name = Self::convert_name2string(dirname);
        let name = name.as_str();
        let meta = guard.create_file(dirid, name, FileMetaType::DIR).await.unwrap();
        let fattr = Self::convert_fattr3(&meta);
        Ok((meta.id.unwrap() as fileid3, fattr))
    }

    async fn remove(&self, dirid: fileid3, filename: &filename3) -> Result<(), nfsstat3> {
        // info!("remove: {}", dirid);
        let guard = self.vfs.write().await;
        let name = Self::convert_name2string(filename);
        let name = name.as_str();
        guard.del_file_sync(dirid, name).await.unwrap();
        Ok(())
    }

    async fn rename(&self, from_dirid: fileid3, from_filename: &filename3, to_dirid: fileid3, to_filename: &filename3) -> Result<(), nfsstat3> {
        // info!("rename: {} {}", from_dirid,  to_dirid);
        let guard = self.vfs.write().await;
        let from_name = Self::convert_name2string(from_filename);
        let from_name = from_name.as_str();
        let to_filename = Self::convert_name2string(to_filename);
        let to_filename = to_filename.as_str();
        guard.rename(from_dirid, from_name, to_dirid, to_filename).await.unwrap();
        Ok(())
    }

    async fn readdir(&self, dirid: fileid3, start_after: fileid3, max_entries: usize) -> Result<ReadDirResult, nfsstat3> {
        // info!("readdir: {}", dirid);
        let guard = self.vfs.read().await;
        let result = guard.list_by_parent_page(dirid, start_after, max_entries).await;
        if let Err(x) = result {
            error!("readdir: {}", x);
            return Err(nfsstat3::NFS3ERR_NOENT);
        }
        let (list, end) = result.unwrap();

        let mut vec = Vec::new();
        for item in list {
            let fattr3 = Self::convert_fattr3(&item);
            let filename = Self::convert_name2filename(&item.name);
            let entry = DirEntry {
                fileid: item.id.unwrap() as fileid3,
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

    async fn symlink(&self, _dirid: fileid3, _linkname: &filename3, _symlink: &nfspath3, _attr: &sattr3) -> Result<(fileid3, fattr3), nfsstat3> {
        info!("symlink: {}", _dirid);
        todo!()
    }

    async fn readlink(&self, _id: fileid3) -> Result<nfspath3, nfsstat3> {
        info!("readlink: {}", _id);
        todo!()
    }
}