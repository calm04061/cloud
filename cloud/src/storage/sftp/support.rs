use crate::error::ErrorInfo;
use crate::storage::sftp::vo::HostUser;

impl From<String> for HostUser {
    fn from(value: String) -> Self {
        serde_json::from_str(value.as_str()).unwrap()
    }
}

impl From<async_ssh2_lite::Error> for ErrorInfo {
    fn from(value: async_ssh2_lite::Error) -> Self {
        ErrorInfo::OTHER(20, value.to_string())
    }
}

impl From<std::io::Error> for ErrorInfo {
    fn from(value: std::io::Error) -> Self {
        ErrorInfo::OTHER(20, value.to_string())
    }
}