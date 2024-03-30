use log::info;
use rbatis::rbdc::db::ExecResult;
use rbatis::rbdc::DateTime;
use rbatis::RBatis;
use rbs::Value;

use api::ResponseResult;
use persistence::{CloudFileBlock, FileStatus};

pub struct CloudFileBlockManager {
    batis: RBatis,
}

impl CloudFileBlockManager {
    pub fn new(batis: RBatis) -> CloudFileBlockManager {
        CloudFileBlockManager {
            batis,
        }
    }
    pub async fn update_by_status(&self, block: &CloudFileBlock, status: FileStatus) -> ResponseResult<u64> {
        Ok(CloudFileBlock::update_by_status(&self.batis.clone(), block, block.id.unwrap(), status.into()).await?.rows_affected)
    }
    pub async fn select_by_file_block_id(&self, file_block_id: i32) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(CloudFileBlock::select_by_column(&self.batis.clone(), "file_block_id", file_block_id)
            .await?)
    }
    pub async fn update(&self, block: &CloudFileBlock) -> ResponseResult<u64> {
        Ok(CloudFileBlock::update_by_column(&self.batis.clone(), block, "id")
            .await
            ?.rows_affected)
    }
    pub async fn insert(&self, block: &CloudFileBlock) -> ResponseResult<ExecResult> {
        Ok(CloudFileBlock::insert(&self.batis.clone(), block).await?)
    }
    pub async fn delete_by_id(&self, id: i32) -> ResponseResult<u64> {
        Ok(CloudFileBlock::delete_by_column(&self.batis.clone(), "id", id)
            .await?.rows_affected)
    }
    pub async fn select_to_upload(&self) -> ResponseResult<Vec<CloudFileBlock>> {
        let cloud_file_block = CloudFileBlock::select_to_upload(&self.batis.clone()).await?;
        info!("{} block to upload", cloud_file_block.len());
        Ok(cloud_file_block)
    }
    pub async fn select_by_status(&self, status: FileStatus, update_time: DateTime) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(CloudFileBlock::select_by_status(&self.batis.clone(), status.into(), update_time).await?)
    }
    pub async fn select_by_status_limit(&self, status: FileStatus, size: usize) -> ResponseResult<Vec<CloudFileBlock>> {
        Ok(CloudFileBlock::select_by_status_limit(&self.batis.clone(), status.into(), size).await?)
    }
    pub async fn query_block_need_re_balance(&self, size: i32) -> ResponseResult<Vec<TempRow>> {
        let rbatis = &self.batis.clone();
        let sql = r#"
        select id as file_block_id from (
            select fbm.id, count(cfb.id) size from file_block_meta fbm
            left join cloud_file_block cfb on fbm.id = cfb.file_block_id
            left join file_meta fm on fbm.file_meta_id = fm.id
            where fm.deleted = 0
            group by cfb.id
        ) where size < ? limit 50
        "#;
        Ok(rbatis.query_decode::<Vec<TempRow>>(sql, vec![Value::U32(size as u32)])
            .await?)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TempRow {
    pub file_block_id: Option<i32>,
}
