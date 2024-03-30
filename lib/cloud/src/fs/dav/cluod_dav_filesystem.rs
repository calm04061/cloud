use std::sync::Arc;

use dav_server::davpath::DavPath;
use dav_server::fs::{BoxCloneFs, DavDirEntry, DavFile, DavFileSystem, DavMetaData, FsError, FsFuture, FsStream, OpenOptions, ReadDirMeta};
use futures_util::FutureExt;
use log::info;

use crate::fs::dav::meta::{CloudDavDirEntry, CloudDavFile, CloudFsMetaData};
use crate::fs::vfs::VirtualFileSystem;

pub(crate) const DAV_PREFIX: &str = "/dav";

pub(crate) struct CloudDavFsInner {
    pub(crate) fs: VirtualFileSystem,
    // pub(crate) fs_access_guard: Option<Box<dyn Fn() -> Box<dyn Any> + Send + Sync + 'static>>,
    pub(crate) prefix: String,
}

pub(crate) struct CloudDavFs {
    pub(crate) inner: Arc<CloudDavFsInner>,
}

impl CloudDavFs {
    pub(crate) fn new(cache_file: &str, prefix: &str) -> Self {
        CloudDavFs {
            inner: Arc::new(CloudDavFsInner {
                fs: VirtualFileSystem::new(cache_file),
                // fs_access_guard: None,
                prefix: prefix.to_string(),
            }),
        }
    }
    fn fs_path(&self, path: &DavPath) -> String {
        let full_path = path.to_string();
        let full_path = full_path.as_str();
        let full_path = &full_path[self.inner.prefix.len()..];
        full_path.to_string()
    }
}

impl BoxCloneFs for CloudDavFs {
    fn box_clone(&self) -> Box<dyn DavFileSystem> {
        Box::new(CloudDavFs {
            inner: self.inner.clone(),
        })
    }
}

impl DavFileSystem for CloudDavFs {
    fn open<'a>(&'a self, path: &'a DavPath, options: OpenOptions) -> FsFuture<Box<dyn DavFile>> {
        async move {
            let full_path = self.fs_path(path);
            info!("FS: open {:?},options:{:?}", full_path,options);
            let meta;
            if options.create {
                let result = self.inner.fs.file_info_by_path(full_path.as_str()).await;
                if let Err(_e) = result {
                    meta = self.inner.fs.create_path_file(full_path.as_str()).await.unwrap();
                } else {
                    meta = result.unwrap();
                }
            } else {
                let result = self.inner.fs.file_info_by_path(full_path.as_str()).await;
                if let Err(_e) = result {
                    return Err(FsError::NotFound);
                }
                meta = result.unwrap();
            }
            let data = CloudDavFile::new(meta, self.inner.fs.clone());
            Ok(Box::new(data) as Box<dyn DavFile>)
        }.boxed()
    }

    fn read_dir<'a>(&'a self, path: &'a DavPath, _meta: ReadDirMeta) -> FsFuture<FsStream<Box<dyn DavDirEntry>>> {
        async move {
            let full_path = self.fs_path(path);
            let result = self.inner.fs.file_info_by_path(full_path.as_str()).await;
            let result = self.inner.fs.list_by_parent(result.unwrap().id.unwrap()).await.unwrap();

            let mut v: Vec<Box<dyn DavDirEntry>> = Vec::new();
            for file in result {
                v.push(Box::new(CloudDavDirEntry::new(file)));
            }
            let stream = futures_util::stream::iter(v.into_iter());
            Ok(Box::pin(stream) as FsStream<Box<dyn DavDirEntry>>)
        }
            .boxed()
    }

    fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<Box<dyn DavMetaData>> {
        async move {
            let full_path = self.fs_path(path);
            let result = self.inner.fs.file_info_by_path(full_path.as_str()).await;
            if let Err(_e) = result {
                return Err(FsError::NotFound);
            }
            let meta = result.unwrap();
            let data = CloudFsMetaData::new(meta);
            Ok(Box::new(data) as Box<dyn DavMetaData>)
        }.boxed()
    }

    fn create_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        async move {
            let full_path = self.fs_path(path);
            self.inner.fs.create_dir(full_path.as_str()).await.ok();
            Ok(())
        }.boxed()
    }

    fn remove_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        self.remove_file(path)
    }

    fn remove_file<'a>(&'a self, path: &'a DavPath) -> FsFuture<()> {
        async move {
            let full_path = self.fs_path(path);
            info!("FS: remove file {:?}", full_path);
            self.inner.fs.delete_one_file(full_path.as_str()).await.ok();
            Ok(())
        }.boxed()
    }

    fn rename<'a>(&'a self, from: &'a DavPath, to: &'a DavPath) -> FsFuture<()> {
        async move {
            let from_full_path = self.fs_path(from);
            let to_full_path = self.fs_path(to);
            self.inner.fs.rename_path(from_full_path.as_str(), to_full_path.as_str()).await.ok();
            Ok(())
        }.boxed()
    }
}
