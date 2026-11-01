use std::sync::Arc;

use api::error::ErrorInfo;
use dav_server::davpath::DavPath;
use dav_server::fs::{DavDirEntry, DavFile, DavFileSystem, DavMetaData, FsError, FsFuture, FsStream, OpenOptions, ReadDirMeta};
use futures_util::FutureExt;
use log::{error, info};

use crate::fs::dav::meta::{CloudDavDirEntry, CloudDavFile, CloudFsMetaData};
use crate::fs::vfs::VirtualFileSystem;

pub(crate) const DAV_PREFIX: &str = "/dav";

pub(crate) struct CloudDavFsInner {
    pub(crate) fs: VirtualFileSystem,
    // pub(crate) fs_access_guard: Option<Box<dyn Fn() -> Box<dyn Any> + Send + Sync + 'static>>,
    pub(crate) prefix: String,
}
#[derive(Clone)]
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
        if full_path.len() < self.inner.prefix.len() || !full_path.starts_with(self.inner.prefix.as_str()) {
            return full_path.to_string();
        }
        full_path[self.inner.prefix.len()..].to_string()
    }
}

pub(crate) fn to_fs_error(e: ErrorInfo) -> FsError {
    error!("dav error: {}", e);
    match e {
        ErrorInfo::FileNotFound(_) | ErrorInfo::NoneCloudFileId(_) => FsError::NotFound,
        ErrorInfo::FileAlreadyExist(_) => FsError::Exists,
        _ => FsError::GeneralFailure,
    }
}

// impl BoxCloneFs for CloudDavFs {
//     fn box_clone(&self) -> Box<dyn DavFileSystem> {
//         Box::new(CloudDavFs {
//             inner: self.inner.clone(),
//         })
//     }
// }

impl DavFileSystem for CloudDavFs {
    fn open<'a>(&'a self, path: &'a DavPath, options: OpenOptions) -> FsFuture<'a, Box<dyn DavFile>> {
        async move {
            let full_path = self.fs_path(path);
            info!("FS: open {:?},options:{:?}", full_path,options);
            let meta = match self.inner.fs.file_info_by_path(full_path.as_str()).await {
                Ok(meta) => meta,
                Err(e) => {
                    if !options.create {
                        return Err(to_fs_error(e));
                    }
                    self.inner.fs.create_path_file(full_path.as_str()).await
                        .map_err(to_fs_error)?
                }
            };
            let id = meta.id;
            let data = CloudDavFile::new(meta, self.inner.fs.clone());
            info!("FS: {:?} opened {:?},options:{:?}", id, full_path, options);
            Ok(Box::new(data) as Box<dyn DavFile>)
        }.boxed()
    }

    fn read_dir<'a>(&'a self, path: &'a DavPath, _meta: ReadDirMeta) -> FsFuture<'a, FsStream<Box<dyn DavDirEntry>>> {
        async move {
            let full_path = self.fs_path(path);
            let parent = self.inner.fs.file_info_by_path(full_path.as_str()).await
                .map_err(to_fs_error)?;
            let Some(parent_id) = parent.id else {
                return Err(FsError::NotFound);
            };
            let result = self.inner.fs.list_by_parent(parent_id as u64).await
                .map_err(to_fs_error)?;

            let mut v: Vec<Result<Box<dyn DavDirEntry>, FsError>> = Vec::new();
            for file in result {
                v.push(Ok(Box::new(CloudDavDirEntry::new(file))));
            }
            let stream = futures_util::stream::iter(v.into_iter());
            Ok(Box::pin(stream) as FsStream<Box<dyn DavDirEntry>>)
        }
            .boxed()
    }

    fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, Box<dyn DavMetaData>> {
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

    fn create_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        async move {
            let full_path = self.fs_path(path);
            self.inner.fs.create_dir(full_path.as_str()).await
                .map_err(to_fs_error)?;
            Ok(())
        }.boxed()
    }

    fn remove_dir<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        self.remove_file(path)
    }

    fn remove_file<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, ()> {
        async move {
            let full_path = self.fs_path(path);
            info!("FS: remove file {:?}", full_path);
            self.inner.fs.delete_one_file(full_path.as_str()).await
                .map_err(to_fs_error)?;
            Ok(())
        }.boxed()
    }

    fn rename<'a>(&'a self, from: &'a DavPath, to: &'a DavPath) -> FsFuture<'a, ()> {
        async move {
            let from_full_path = self.fs_path(from);
            let to_full_path = self.fs_path(to);
            self.inner.fs.rename_path(from_full_path.as_str(), to_full_path.as_str()).await
                .map_err(to_fs_error)?;
            Ok(())
        }.boxed()
    }
}
