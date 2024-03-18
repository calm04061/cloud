use rbatis::rbdc::Error;
use crate::error::ErrorInfo;

impl From<Error> for ErrorInfo{
    fn from(value: Error) -> Self {
        ErrorInfo::new(1,value.to_string().as_str())
     }
}