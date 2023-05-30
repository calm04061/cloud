use crate::database::meta::CloudType;
use serde_derive::Deserialize;
use crate::domain::table::tables::CloudMeta;

#[derive(Deserialize, Clone)]
pub struct CloudMetaVo {
    pub name: String,
    pub cloud_type: CloudType,
}

impl From<CloudMetaVo> for CloudMeta {
    fn from(meta: CloudMetaVo) -> Self {
        let cloud_type:i8=meta.cloud_type.into();
        CloudMeta {
            id: None,
            name: meta.name,
            token: None,
            last_work_time: None,
            data_root: None,
            status: 0,
            deleted: 0,
            cloud_type,
            total_quota: None,
            used_quota: None,
            remaining_quota: None,
            extra: None,
            expires_in: None,
        }
    }
}
