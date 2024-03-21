use std::collections::HashMap;

use actix_web::{get, Responder};
use strum::IntoEnumIterator;

use persistence::CloudType;

use crate::web::common::WebResult;

#[get("/support/cloud/types")]
pub(crate) async fn cloud_types() -> impl Responder {
    let mut result = vec![];
    for cloud in CloudType::iter() {
        let id: i8 = cloud.into();
        let id = id.to_string();
        let name: String = (&cloud).into();
        add_item(&mut result, id, name);
    }
    WebResult::actix_web_json_result(&Some(result))
}

fn add_item<'a>(result: &mut Vec<HashMap<&str, String>>, id: String, name: String) {
    let mut map = HashMap::new();
    map.insert("id", id);
    map.insert("name", name);
    result.push(map);
}