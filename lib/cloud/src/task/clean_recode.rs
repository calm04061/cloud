use log::error;
use service::CONTEXT;

pub(crate) async fn clean_recode() {
    let result = CONTEXT.cloud_file_block_manager.clean_deleted_block().await;
    if let Err(e) = result {
        error!("{}",e);
    }
    let result = CONTEXT.file_block_meta_manager.clean_file_meta().await;
    if let Err(e) = result {
        error!("{}",e);
    }
}