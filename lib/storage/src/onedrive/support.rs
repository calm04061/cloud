use api::ResponseResult;
use persistence::CloudMeta;

use crate::onedrive::vo::{AuthorizationToken, DriveItem, OneDriveQuota, OneDriveUser};
use crate::storage::{CreateResponse, FileInfo, Quota, TokenProvider, User};

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

impl From<DriveItem> for CreateResponse {
    fn from(item: DriveItem) -> Self {
        CreateResponse {
            encrypt_mode: "".to_string(),
            file_id: item.id,
            file_name: item.name,
            file_type: "".to_string(),
        }
    }
}

impl From<DriveItem> for FileInfo {
    fn from(item: DriveItem) -> Self {
        FileInfo {
            create_at: None,
            creator_id: None,
            creator_name: None,
            creator_type: None,
            domain_id: None,
            drive_id: None,
            encrypt_mode: None,
            ex_fields_info: None,
            file_id: item.id,
            path: None,
            hidden: None,
            last_modifier_id: None,
            last_modifier_name: None,
            last_modifier_type: None,
            name: item.name,
            revision_id: None,
            starred: false,
            crc64_hash: None,
            content_hash: None,
            content_hash_name: None,
            download_url: None,
            url: None,
            thumbnail: None,
            image_media_metadata: None,
            category: None,
            file_type: "".to_string(),
            updated_at: None,
            user_meta: None,
            labels: None,
            upload_id: None,
            status: None,
            punish_flag: None,
            parent_file_id: None,
        }
    }
}

impl TokenProvider<AuthorizationToken> for CloudMeta {
    fn get_token(&self) -> ResponseResult<AuthorizationToken> {
        let auth_option = self.auth.clone();
        let token = auth_option.unwrap();
        Ok(serde_json::from_str(token.as_str()).unwrap())
    }
}