use std::cmp::min;
use std::collections::HashMap;
use dotenv_codegen::dotenv;
use log::info;
use rand::random;
use rbs::Value;
use persistence::{CloudFileBlock, CloudMeta, MetaStatus};
use crate::pool;

pub(crate) async fn re_balance() {
    info!("start re balance");
    let max_shard: &str = dotenv!("MAX_NUMBER_OF_SHARD");
    let max_shard: usize = max_shard.parse().unwrap();
    let cloud_metas = CloudMeta::quota_random(pool!(), MetaStatus::Enable.into(), max_shard as i32).await.unwrap();

    let cloud_meta_size = cloud_metas.len();
    let size = min(cloud_meta_size, max_shard);
    let file_block_id_rows = pool!().query_decode::<Vec<TempRow>>("select file_block_id from (select cfb.file_block_id, count(cfb.id) size from file_block_meta fbm left join cloud_file_block cfb on fbm.id = cfb.file_block_id group by cfb.file_block_id ) where size < ? limit 50", vec![Value::U32(size as u32)])
        .await.unwrap();

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
        let mut cloud_file_blocks = CloudFileBlock::select_by_column(pool!(), "file_block_id", file_block_id).await.unwrap();

        let mut remove_index = Vec::new();
        let mut using_cloud_id = Vec::new();

        for (index, row) in cloud_file_blocks.iter().enumerate() {
            let option = id2cloud.get(&row.cloud_meta_id);//移除非法的cloud_file_block
            if option.is_none() {
                CloudFileBlock::delete_by_column(pool!(), "id", file_block_id)
                    .await.unwrap();
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
                CloudFileBlock::insert(pool!(), &block).await.unwrap();
                using_cloud_id.push(cloud_id);
                break;
            }
            info!("re balance file_block_id:{}", file_block_id);
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct TempRow {
    file_block_id: Option<i32>,
}
