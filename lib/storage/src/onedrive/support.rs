use api::error::ErrorInfo;
use crate::model::{CreateResponse, FileInfo, Quota, User};
use api::ResponseResult;
use persistence::meta::CloudMeta;

use crate::onedrive::vo::{AuthorizationToken, DriveItem, OneDriveQuota, OneDriveUser};
use crate::storage::TokenProvider;

impl From<OneDriveQuota> for Quota {
    fn from(one: OneDriveQuota) -> Self {
        Quota {
            total: one.total as i64,
            used: one.used as i64,
            remaining: one.remaining as i64,
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
        if let None = auth_option {
            return  Err(ErrorInfo::Http402("".to_string()));
        }
        let auth = auth_option.unwrap();
        Ok(serde_json::from_str(&auth)?)
    }
}