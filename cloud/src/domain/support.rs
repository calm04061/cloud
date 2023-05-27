use crate::database::meta::FileStatus;
use crate::domain::table::tables::CloudFileBlock;
use rbatis::rbdc::datetime::DateTime;

impl CloudFileBlock {
    pub(crate) fn init(file_block_meta_id: i64, cloud_meta_id: i32) -> Self {
        CloudFileBlock {
            id: None,
            file_block_id: file_block_meta_id,
            cloud_meta_id,
            cloud_file_id: None,
            cloud_file_hash: None,
            status: FileStatus::Init.into(),
            deleted: 0,
            create_time: DateTime::now(),
            update_time: DateTime::now(),
        }
    }
}
