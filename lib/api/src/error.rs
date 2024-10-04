use std::error::Error;
use std::fmt::{Display, Formatter, Write};

use actix_web::body::BoxBody;
use actix_web::http::header;
use actix_web::http::header::TryIntoHeaderValue;
use actix_web::{error, HttpResponse};
use bytes::BytesMut;

#[derive(Debug, PartialEq, Clone, serde_derive::Serialize)]
pub enum ErrorInfo {
    Retry,
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
}

unsafe impl Send for ErrorInfo {}

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (code, message) = match self {
            ErrorInfo::Retry => (3000, String::from("retry")),
            ErrorInfo::FileNotFound(f) => (4000, f.clone()),
            ErrorInfo::FileAlreadyExist(f) => (4001, f.clone()),
            ErrorInfo::Http302(url) => (5302, url.clone()),
            ErrorInfo::Http401(url) => (5401, String::from(url.as_str())),
            ErrorInfo::Http402(msg) => (5402, String::from(msg.as_str())),
            ErrorInfo::Http404(url) => (5404, String::from(url.as_str())),
            ErrorInfo::Http(code) => (5000 + code, String::from("")),
            ErrorInfo::OTHER(code, msg) => (6000 + code, msg.clone()),
            ErrorInfo::NoneCloudFileId(cloud_meta_id) => (7000, format!("未找到云文件ID:{}", cloud_meta_id)),
            ErrorInfo::NoneCloudMeta(cloud_meta_id) => (7000, format!("云配置：{},没有找到", cloud_meta_id)),
            // _ => (8000, "系统错误".to_string()),
        };
        let string = format!("{}:{}", code, message);
        f.write_str(string.as_str())
    }
}

impl Error for ErrorInfo {}

impl error::ResponseError for ErrorInfo {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());

        let mut buf = BytesMut::new();
        let result = serde_json::to_string(&self).unwrap();
        buf.write_str(result.as_str()).ok();
        // let _ = write!(helpers::MutWriter(&mut buf), "{}", result);
        let mime = mime::APPLICATION_JSON.try_into_value().unwrap();

        res.headers_mut().insert(header::CONTENT_TYPE, mime);

        res.set_body(BoxBody::new(buf))
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