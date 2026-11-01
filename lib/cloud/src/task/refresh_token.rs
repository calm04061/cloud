use api::ResponseResult;
use log::error;
use service::CONTEXT;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::storage_facade;

pub(crate) async fn refresh_token() -> ResponseResult<()> {
    let current_time = SystemTime::now();
    let now = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let cloud_metas = CONTEXT.cloud_meta_manager.query_token_timeout_cloud_meta(now).await?;
    for cloud in cloud_metas {
        let result = storage_facade().refresh_token(cloud.id.unwrap()).await;
        if let Err(e) = result {
            error!("refresh token error: {}", e);
        }
    }
    Ok(())
}