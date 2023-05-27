use crate::storage::ali::vo::DriveCapacity;
use crate::storage::storage::Quota;

impl From<DriveCapacity> for Quota {
    fn from(baidu: DriveCapacity) -> Self {
        Quota {
            total: baidu.total_size,
            used: baidu.used_size,
            remaining: baidu.total_size - baidu.used_size,
        }
    }
}
