use serde_derive::Deserialize;
use persistence::{CloudMeta, CloudType};

#[derive(Deserialize, Clone)]
pub struct CloudMetaVo {
    pub name: String,
    pub cloud_type: CloudType,
    pub auth: Option<String>,
    pub data_root: Option<String>,
}

impl From<CloudMetaVo> for CloudMeta {
    fn from(meta: CloudMetaVo) -> Self {
        let cloud_type: i8 = meta.cloud_type.into();
        CloudMeta {
            id: None,
            name: meta.name,
            auth: meta.auth,
            last_work_time: None,
            data_root: meta.data_root,
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
