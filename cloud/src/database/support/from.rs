use crate::database::meta::cloud::MetaStatus;
use crate::database::meta::cloud::MetaStatus::{
    Disabled, Enable, InvalidRefresh, WaitDataRoot, WaitInit,
};
use crate::database::meta::FileMetaType::{DIR, FILE, SYMLINK};
use crate::database::meta::{CloudType, FileMetaType, FileStatus};

impl From<i8> for MetaStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => WaitInit,
            1 => WaitDataRoot,
            2 => Enable,
            3 => InvalidRefresh,
            4 => Disabled,
            _ => Disabled,
        }
    }
}

impl From<FileStatus> for i8 {
    fn from(status: FileStatus) -> Self {
        match status {
            FileStatus::Init => 1,
            FileStatus::Uploading => 2,
            FileStatus::UploadSuccess => 3,
            FileStatus::UploadFail => 4,
            FileStatus::FileNotExist => 5,
            FileStatus::FileReadError => 6,
            FileStatus::WaitClean => 7,
        }
    }
}

impl From<&MetaStatus> for i8 {
    fn from(value: &MetaStatus) -> Self {
        match value {
            WaitInit => 0,
            WaitDataRoot => 1,
            Enable => 2,
            InvalidRefresh => 3,
            Disabled => 4,
        }
    }
}

impl From<MetaStatus> for i8 {
    fn from(value: MetaStatus) -> Self {
        match value {
            WaitInit => 0,
            WaitDataRoot => 1,
            Enable => 2,
            InvalidRefresh => 3,
            Disabled => 4,
        }
    }
}

impl FileMetaType {
    pub fn is_file(code: i8) -> bool {
        return code == FILE.get_code();
    }
    pub fn is_dir(code: i8) -> bool {
        return code == DIR.get_code();
    }
    pub fn get_code(&self) -> i8 {
        return if self == &FILE { 1 } else { 2 };
    }
}

impl From<i8> for FileMetaType {
    fn from(value: i8) -> Self {
        match value {
            1 => FILE,
            2 => DIR,
            _ => FILE,
        }
    }
}
impl From<i32> for FileMetaType {
    fn from(value: i32) -> Self {
        match value {
            1 => FILE,
            2 => DIR,
            3 => SYMLINK,
            _ => FILE
        }
    }
}

impl From<&FileMetaType> for i32 {
    fn from(value: &FileMetaType) -> Self {
        match value {
            FILE => 1,
            DIR => 2,
            SYMLINK => 3
        }
    }
}

impl From<i8> for CloudType {
    fn from(value: i8) -> Self {
        match value {
            1 => CloudType::AliYun,
            2 => CloudType::Baidu,
            3 => CloudType::Local,
            _ => CloudType::AliYun,
        }
    }
}

impl From<&CloudType> for i8 {
    fn from(value: &CloudType) -> Self {
        match value {
            CloudType::AliYun => 1,
            CloudType::Baidu => 2,
            CloudType::Local => 3,
        }
    }
}
impl From<CloudType> for i8 {
    fn from(value: CloudType) -> Self {
        return i8::from(&value);
    }
}