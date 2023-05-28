use crate::storage::onedrive::vo::{OneDriveQuota, OneDriveUser};
use crate::storage::storage::{Quota, User};

impl From<OneDriveQuota> for Quota {
    fn from(one: OneDriveQuota) -> Self {
        Quota {
            total: one.total,
            used: one.used,
            remaining: one.remaining,
        }
    }
}
impl From<OneDriveUser> for User {
    fn from(one: OneDriveUser) -> Self {
        User {
            domain_id: None,
            user_id: Some(one.id),
            avatar: None,
            email: None,
            nick_name: Some(one.display_name.clone()),
            phone: None,
            role: None,
            status: None,
            user_name: Some(one.display_name),
            default_drive_id: None,
            deny_change_password_by_self: None,
            need_change_password_next_login: None,
            creator: None,
            created_at: None,
            updated_at: None,
        }
    }
}
