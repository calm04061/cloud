use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AuthorizationToken {
    pub(crate) token_type: String,
    expires_in: u64,
    scope: String,
    pub(crate) access_token: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Drive {
    pub(crate) owner: OneDriveOwner,
    pub(crate) quota: OneDriveQuota,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveQuota {
    deleted: u64,
    file_count: u64,
    pub(crate) remaining: u64,
    state: String,
    pub(crate) total: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveOwner {
    pub(crate) user: OneDriveUser,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveUser {
    pub(crate) id: String,
    pub(crate) display_name: String,
}
