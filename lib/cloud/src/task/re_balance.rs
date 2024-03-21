use std::cmp::min;
use std::collections::HashMap;
use dotenvy_macro::dotenv;

use log::info;
use rand::random;
use api::ResponseResult;

use persistence::{CloudFileBlock, MetaStatus};
use service::CONTEXT;
const MAX_SHARD: &str = dotenv!("MAX_NUMBER_OF_SHARD");

pub(crate) async fn re_balance() -> ResponseResult<()>{
    info!("start re balance");
    let max_shard: usize = MAX_SHARD.parse().unwrap();
    let cloud_metas = CONTEXT.cloud_meta_manager.quota_random(MetaStatus::Enable.into(), max_shard as i32).await;

    let cloud_meta_size = cloud_metas.len();
    let size = min(cloud_meta_size, max_shard);
    let file_block_id_rows = CONTEXT.cloud_file_block_manager.query_block_need_re_balance(size as i32).await;
    // let file_block_id_rows = pool!().query_decode::<Vec<TempRow>>("select file_block_id from (select cfb.file_block_id, count(cfb.id) size from file_block_meta fbm left join cloud_file_block cfb on fbm.id = cfb.file_block_id group by cfb.file_block_id ) where size < ? limit 50", vec![Value::U32(size as u32)])
    //     .await.unwrap();

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
        let mut cloud_file_blocks =CONTEXT.cloud_file_block_manager.select_by_file_block_id( file_block_id).await?;

        let mut remove_index = Vec::new();
        let mut using_cloud_id = Vec::new();

        for (index, row) in cloud_file_blocks.iter().enumerate() {
            let option = id2cloud.get(&row.cloud_meta_id);//移除非法的cloud_file_block
            if option.is_none() {
                CONTEXT.cloud_file_block_manager.delete_by_id(file_block_id).await;
                // CloudFileBlock::delete_by_column(pool!(), "id", file_block_id)
                //     .await.unwrap();
                remove_index.push(index);
                continue;
            }
            using_cloud_id.push(row.cloud_meta_id);
        }
        for index in remove_index {
            cloud_file_blocks.remove(index);
        }
        for _i in cloud_file_blocks.len()..size {
            loop {
                let random = random::<usize>();
                let index = random % cloud_meta_size;
                let cloud_id = cloud_metas[index].id.unwrap();
                if using_cloud_id.len() >= cloud_meta_size {
                    break;
                }
                if using_cloud_id.contains(&cloud_id) {
                    continue;
                }
                let block = CloudFileBlock::init(file_block_id, cloud_id);
                CONTEXT.cloud_file_block_manager.insert(&block).await;
                // CloudFileBlock::insert(pool!(), &block).await.unwrap();
                using_cloud_id.push(cloud_id);
                break;
            }
            info!("re balance file_block_id:{file_block_id}");
        }
    }
    Ok(())
}
