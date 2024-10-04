use actix_web::dev;
use actix_web::http::header;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::web::Json;
use serde_derive::Serialize;

pub(crate) fn add_error_header<B>(
    mut res: dev::ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WebResult<T> {
    message: String,
    code: i32,
    data: Option<T>,
}

impl<T: Clone> WebResult<T> {
    pub fn success(t: T) -> WebResult<T> {
        WebResult {
            message: "".to_string(),
            code: 0,
            data: Some(t),
        }
    }
    pub fn empty() -> WebResult<T> {
        WebResult {
            message: "".to_string(),
            code: 0,
            data: None,
        }
    }
    pub fn fail(code: i32, message: &str) -> WebResult<T> {
        WebResult {
            message: String::from(message),
            code,
            data: None,
        }
    }
    // pub fn result(x: &Option<T>) -> WebResult<T> {
    //     match x {
    //         None => {
    //             WebResult::empty()
    //         }
    //         Some(t) => {
    //             WebResult::success(t.clone())
    //         }
    //     }
    // }

    pub fn actix_web_json_result(x: &Option<T>) -> Json<WebResult<T>> {
        match x {
            None => Json(WebResult::empty()),
            Some(t) => Json(WebResult::success(t.clone())),
        }
    }
}
