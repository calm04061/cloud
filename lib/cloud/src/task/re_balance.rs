use dotenvy;
use std::cmp::min;
use std::collections::HashMap;
use api::ResponseResult;
use rand::random;

use persistence::{CloudFileBlock, MetaStatus, TempCloudFile};
use service::CONTEXT;

fn get_max_shard() -> String {
    dotenvy::var("MAX_NUMBER_OF_SHARD").unwrap_or_else(|_| "3".to_string())
}

pub(crate) async fn re_balance() -> ResponseResult<()> {
    // info!("start re balance");
    let option = CONTEXT.config_manager.info("MAX_NUMBER_OF_SHARD").await;
    let default_max_shard = option.map(|e| e.value).unwrap_or_else(|| get_max_shard());
    let default_max_shard: usize = match default_max_shard.parse() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse MAX_NUMBER_OF_SHARD: {}", e);
            return Err(api::error::ErrorInfo::OTHER(format!("Invalid MAX_NUMBER_OF_SHARD: {}", e)));
        }
    };
    let cloud_metas = CONTEXT.cloud_meta_manager.quota_random(MetaStatus::Enable.into(), default_max_shard as i32).await?;

    let cloud_meta_size = cloud_metas.len();
    let size = min(cloud_meta_size, default_max_shard);
    let file_block_id_rows = CONTEXT.cloud_file_block_manager.query_block_need_re_balance(size as i32).await?;
    // println!("{:?}", file_block_id_rows);
    let mut id2cloud = HashMap::new();
    for cloud in &cloud_metas {
        id2cloud.insert(cloud.id.unwrap(), cloud);
    }
    for row in file_block_id_rows {
        let file_block_id = row.file_block_id;
        if file_block_id == None {
            continue;
        }
        let file_block_id = file_block_id.unwrap();
        let mut cloud_file_blocks = CONTEXT.cloud_file_block_manager.select_by_file_block_id(file_block_id).await?;
        // info!("file block id: {} cloud_block size:{}", file_block_id, cloud_file_blocks.len());

        if cloud_file_blocks.len() > size {
            let _ = clean_cloud_file_block(cloud_file_blocks, size).await;
        } else {
            let mut remove_index = Vec::new();
            let mut using_cloud_id = Vec::new();
            for (index, row) in cloud_file_blocks.iter().enumerate() {
                let option = id2cloud.get(&row.cloud_meta_id); //移除非法的cloud_file_block
                if option.is_none() {
                    CONTEXT.cloud_file_block_manager.delete_by_id(row.id.unwrap()).await?;
                    remove_index.push(index);
                    continue;
                }
                using_cloud_id.push(row.cloud_meta_id);
            }
            remove_index.reverse();
            for index in remove_index {
                cloud_file_blocks.remove(index);
            }
            for _i in cloud_file_blocks.len()..size {
                loop {
                    let random = random::<u32>() as usize;
                    let index = random % cloud_meta_size;
                    let cloud_id = cloud_metas[index].id.unwrap();
                    if using_cloud_id.len() >= cloud_meta_size {
                        break;
                    }
                    if using_cloud_id.contains(&cloud_id) {
                        continue;
                    }
                    let block = CloudFileBlock::init(file_block_id, cloud_id);
                    CONTEXT.cloud_file_block_manager.insert(&block).await?;
                    using_cloud_id.push(cloud_id);
                    break;
                }
                // info!("re balance file_block_id:{file_block_id}");
            }
        }
    }
    Ok(())
}
async fn clean_cloud_file_block(mut cloud_file_blocks: Vec<CloudFileBlock>, size: usize) -> ResponseResult<()>{
    while cloud_file_blocks.len() > size {
        let block = cloud_file_blocks.remove(0);
        if let Some(cloud_file_id) = block.cloud_file_id {
            let temp = TempCloudFile::new(block.cloud_meta_id, cloud_file_id);
            CONTEXT.temp_cloud_file_manager.add(&temp).await?;
        }
        CONTEXT.cloud_file_block_manager.delete_by_id(block.id.unwrap()).await?;
    }
    Ok(())
}