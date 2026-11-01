use std::error::Error;
use std::fmt::{Display, Formatter};

use actix_web::body::BoxBody;
use actix_web::{error, HttpResponse};

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum ErrorInfo {
    Retry,
    NoSuchTable(String),
    FileNotFound(String),
    FileAlreadyExist(String),
    Http302(String),
    Http401(String),
    Http402(String),
    Http404(String),
    NoneCloudFileId(i32),
    NoneCloudMeta(i32),
    Http(i32),
    OTHER(i32, String),
}

impl ErrorInfo {
    pub fn new(code: i32, message: &str) -> Self {
        ErrorInfo::OTHER(code, message.to_string())
    }
    pub fn new_string(code: i32, message: String) -> Self {
        ErrorInfo::OTHER(code, message)
    }

    /// 稳定的业务错误码，所有错误响应体统一为 `{message, code, data}`。
    pub fn code(&self) -> i32 {
        match self {
            ErrorInfo::Retry => 3000,
            ErrorInfo::FileNotFound(_) => 4000,
            ErrorInfo::FileAlreadyExist(_) => 4001,
            ErrorInfo::Http302(_) => 5302,
            ErrorInfo::Http401(_) => 5401,
            ErrorInfo::Http402(_) => 5402,
            ErrorInfo::Http404(_) => 5404,
            ErrorInfo::Http(code) => 5000 + code,
            ErrorInfo::OTHER(code, _) => 6000 + code,
            ErrorInfo::NoneCloudFileId(_) | ErrorInfo::NoneCloudMeta(_) => 7000,
            ErrorInfo::NoSuchTable(_) => 8000,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ErrorInfo::Retry => String::from("retry"),
            ErrorInfo::FileNotFound(f) => f.clone(),
            ErrorInfo::FileAlreadyExist(f) => f.clone(),
            ErrorInfo::Http302(url) => url.clone(),
            ErrorInfo::Http401(url) => url.clone(),
            ErrorInfo::Http402(msg) => msg.clone(),
            ErrorInfo::Http404(url) => url.clone(),
            ErrorInfo::Http(_) => String::from(""),
            ErrorInfo::OTHER(_, msg) => msg.clone(),
            ErrorInfo::NoneCloudFileId(cloud_meta_id) => format!("未找到云文件ID:{}", cloud_meta_id),
            ErrorInfo::NoneCloudMeta(cloud_meta_id) => format!("云配置：{},没有找到", cloud_meta_id),
            ErrorInfo::NoSuchTable(table) => format!("表：{},没有找到", table),
        }
    }
}

unsafe impl Send for ErrorInfo {}

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = format!("{}:{}", self.code(), self.message());
        f.write_str(string.as_str())
    }
}

impl Error for ErrorInfo {}

/// 统一错误响应体，与成功响应 `{message, code, data}` 结构一致。
#[derive(serde_derive::Serialize)]
struct ErrorEnvelope {
    message: String,
    code: i32,
    data: Option<()>,
}

impl error::ResponseError for ErrorInfo {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        match self {
            ErrorInfo::FileNotFound(_) | ErrorInfo::Http404(_) => StatusCode::NOT_FOUND,
            ErrorInfo::FileAlreadyExist(_) => StatusCode::CONFLICT,
            ErrorInfo::Http401(_) | ErrorInfo::Http402(_) => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = ErrorEnvelope {
            message: self.message(),
            code: self.code(),
            data: None,
        };
        HttpResponse::build(self.status_code()).json(body)
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
// impl From<rbatis::rbdc::Error> for ErrorInfo {
//     fn from(value: rbatis::rbdc::Error) -> Self {
//         ErrorInfo::new(1, value.to_string().as_str())
//     }
// }
impl From<sqlx::Error> for ErrorInfo {
    fn from(value: sqlx::Error) -> Self {
        ErrorInfo::new(1, value.to_string().as_str())
    }
}
