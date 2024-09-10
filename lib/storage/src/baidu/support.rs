use api::error::ErrorInfo;
use crate::baidu::vo::{AsyncType, BaiduFileMeta, BaiduOpera, BaiduQuota, BaiduUser, Token};
use crate::model::{FileInfo, Quota, User};
use crate::storage::TokenProvider;
use api::ResponseResult;
use persistence::meta::CloudMeta;

impl From<BaiduUser> for User {
    fn from(baidu: BaiduUser) -> Self {
        User {
            domain_id: None,
            user_id: Some(baidu.baidu_name),
            avatar: baidu.avatar_url,
            email: None,
            nick_name: None,
            phone: None,
            role: None,
            status: None,
            user_name: None,
            default_drive_id: None,
            deny_change_password_by_self: None,
            need_change_password_next_login: None,
            creator: None,
            created_at: None,
            updated_at: None,
        }
    }
}

impl From<BaiduFileMeta> for FileInfo {
    fn from(baidu: BaiduFileMeta) -> Self {
        FileInfo {
            create_at: None,
            creator_id: None,
            creator_name: None,
            creator_type: None,
            domain_id: Some("".to_string()),
            drive_id: Some("".to_string()),
            encrypt_mode: Some("".to_string()),
            ex_fields_info: None,
            file_id: baidu.fs_id.to_string(),
            path: Some(baidu.path),
            hidden: Some(false),
            last_modifier_id: None,
            last_modifier_name: None,
            last_modifier_type: None,
            name: baidu.filename,
            revision_id: Some("".to_string()),
            starred: false,
            status: Some("".to_string()),
            file_type: "".to_string(),
            updated_at: Some("".to_string()),
            user_meta: None,
            labels: None,
            upload_id: None,
            parent_file_id: None,
            crc64_hash: None,
            content_hash: None,
            content_hash_name: None,
            download_url: baidu.dlink,
            url: None,
            thumbnail: None,
            image_media_metadata: None,
            category: None,
            punish_flag: None,
        }
    }
}

impl From<BaiduOpera> for &str {
    fn from(opera: BaiduOpera) -> Self {
        match opera {
            BaiduOpera::Delete => "delete", // BaiduOpera::Copy => { "copy" }
                                            // BaiduOpera::Move => { "move" }
                                            // BaiduOpera::Rename => { "rename" }
        }
    }
}

impl From<AsyncType> for &str {
    fn from(opera: AsyncType) -> Self {
        match opera {
            AsyncType::Async => "2", // AsyncType::SelfAdaption => { "1" }
                                     // AsyncType::Sync => { "0" }
        }
    }
}

impl From<BaiduQuota> for Quota {
    fn from(baidu: BaiduQuota) -> Self {
        Quota {
            total: baidu.total as i64,
            used: baidu.used as i64,
            remaining: (baidu.total - baidu.used) as i64,
        }
    }
}

impl TokenProvider<Token> for CloudMeta{
    fn get_token(&self) -> ResponseResult<Token> {
        let auth_option = self.auth.clone();
        if let None = auth_option {
           return  Err(ErrorInfo::Http402("".to_string()));
        }
        let auth = auth_option.unwrap();
        Ok(serde_json::from_str(&auth)?)
    }
}