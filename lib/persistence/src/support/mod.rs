use crate::FileStatus;
use api::error::ErrorInfo;

mod cloud_file_block;
mod config;
mod event_message;
mod file_block_meta;
mod file_meta;
mod cloud_meta;
mod cloud_type;
mod file_meta_type;
mod meta_status;
pub mod application_config;
mod user;

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
            FileStatus::Cleaning => 8,
            FileStatus::Cleaned => 9,
            FileStatus::CleanFail => 10,
        }
    }
}
impl TryFrom<i8> for FileStatus {
    type Error = ErrorInfo;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(FileStatus::Init),
            2 => Ok(FileStatus::Uploading),
            3 => Ok(FileStatus::UploadSuccess),
            4 => Ok(FileStatus::UploadFail),
            5 => Ok(FileStatus::FileNotExist),
            6 => Ok(FileStatus::FileReadError),
            7 => Ok(FileStatus::WaitClean),
            8 => Ok(FileStatus::Cleaning),
            9 => Ok(FileStatus::Cleaned),
            10 => Ok(FileStatus::CleanFail),
            _ => Err(ErrorInfo::new(1,"")),
        }
    }
}
