use std::collections::HashMap;

use actix_web::{get, Responder};

use crate::web::common::WebResult;

#[get("/support/cloud/types")]
pub(crate) async fn cloud_types() -> impl Responder {
    let mut result = vec![];
    add_item(&mut result, "1", "阿里云盘");
    add_item(&mut result, "2", "百度云盘");
    add_item(&mut result, "3", "本地磁盘");
    add_item(&mut result, "4", "OneDriver");
    #[cfg(not(windows))]
    add_item(&mut result, "5", "SFTP");
    WebResult::actix_web_json_result(&Some(result))
}

fn add_item<'a>(result: &mut Vec<HashMap<&str, &'a str>>, id: &'a str, name: &'a str) {
    let mut map = HashMap::new();
    map.insert("id", id);
    map.insert("name", name);
    result.push(map);
}