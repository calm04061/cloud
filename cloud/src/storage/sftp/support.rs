use crate::storage::sftp::vo::HostUser;

impl From<String> for HostUser {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap()
    }
}
