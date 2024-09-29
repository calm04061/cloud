use chrono::Utc;
use crate::{FileStatus, TempCloudFile};

impl TempCloudFile {
    pub fn new(cloud_meta_id: i32, cloud_file_id: String) -> TempCloudFile {
        TempCloudFile {
            id: None,
            cloud_meta_id,
            cloud_file_id: Some(cloud_file_id),
            status: FileStatus::WaitClean.into(),
            create_time: Utc::now(),
            update_time: Utc::now(),
        }
    }
}