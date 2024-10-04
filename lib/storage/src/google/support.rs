use crate::model::{FileInfo, Quota};
use api::error::ErrorInfo;
use api::ResponseResult;
use persistence::meta::CloudMeta;

use crate::google::vo::{AuthorizationToken, GoogleDriveFile, GoogleDriveQuota};
use crate::storage::TokenProvider;

impl From<GoogleDriveQuota> for Quota {
    fn from(one: GoogleDriveQuota) -> Self {
        let limit = one.limit.parse::<i64>().unwrap();
        let usage = one.usage.parse::<i64>().unwrap();
        Quota {
            total: limit,
            used: usage,
            remaining: (limit - usage),
        }
    }
}

// impl From<GoogleDriveUser> for User {
//     fn from(one: GoogleDriveUser) -> Self {
//         User {
//             domain_id: None,
//             user_id: Some(one.id),
//             avatar: None,
//             email: None,
//             nick_name: Some(one.display_name.clone()),
//             phone: None,
//             role: None,
//             status: None,
//             user_name: Some(one.display_name),
//             default_drive_id: None,
//             deny_change_password_by_self: None,
//             need_change_password_next_login: None,
//             creator: None,
//             created_at: None,
//             updated_at: None,
//         }
//     }
// }

impl From<GoogleDriveFile> for FileInfo {
    fn from(item: GoogleDriveFile) -> Self {
        FileInfo {
            create_at: None,
            creator_id: None,
            creator_name: None,
            creator_type: None,
            domain_id: None,
            drive_id: None,
            encrypt_mode: None,
            ex_fields_info: None,
            file_id: item.id.unwrap(),
            path: None,
            hidden: None,
            last_modifier_id: None,
            last_modifier_name: None,
            last_modifier_type: None,
            name: item.name.unwrap(),
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
            return Err(ErrorInfo::Http402("".to_string()));
        }
        let auth = auth_option.unwrap();
        Ok(serde_json::from_str(&auth)?)
    }
}

impl GoogleDriveFile {
    pub(crate) fn meta(name: &str, parents: Vec<String>) -> GoogleDriveFile {
        GoogleDriveFile {
            kind: None,
            drive_id: None,
            id: None,
            mime_type: None,
            name: Some(name.to_string()),
            web_content_link: None,
            parents: Some(parents),
        }
    }
    pub(crate) fn new_folder(name: &str, parents: Vec<String>) -> GoogleDriveFile {
        GoogleDriveFile {
            kind: None,
            drive_id: None,
            id: None,
            mime_type: Some("application/vnd.google-apps.folder".to_string()),
            name: Some(name.to_string()),
            web_content_link: None,
            parents: Some(parents),
        }
    }
}