use log::info;
use rbatis::RBatis;
use rbatis::rbdc::DateTime;
use rbatis::rbdc::db::ExecResult;
use rbs::Value;
use api::ResponseResult;
use persistence::CloudFileBlock;

pub struct CloudFileBlockManager {
    batis: RBatis,
}

impl CloudFileBlockManager {
    pub(crate) fn new(batis: RBatis) -> CloudFileBlockManager {
        CloudFileBlockManager {
            batis,
        }
    }
    pub async fn update_by_status(&self, block: &CloudFileBlock, id: i32, status: i8) -> ResponseResult<ExecResult> {
        Ok(CloudFileBlock::update_by_status(&self.batis.clone(), block, id, status).await?)
    }
    pub async fn select_by_file_block_id(&self, file_block_id: i32) -> Vec<CloudFileBlock> {
        CloudFileBlock::select_by_column(&self.batis.clone(), "file_block_id", file_block_id)
            .await
            .unwrap()
    }
    pub async fn update(&self, block: &CloudFileBlock) {
        CloudFileBlock::update_by_column(&self.batis.clone(), block, "id")
            .await
            .unwrap();
    }
    pub async fn insert(&self, block: &CloudFileBlock) {
        CloudFileBlock::insert(&self.batis.clone(), block).await.unwrap();
    }
    pub async fn delete_by_id(&self, id: i32) {
        CloudFileBlock::delete_by_column(&self.batis.clone(), "id", id)
            .await.unwrap();
    }
    pub async fn select_to_upload(&self) -> Vec<CloudFileBlock> {
        let cloud_file_block = CloudFileBlock::select_to_upload(&self.batis.clone()).await.unwrap();
        info!("{} block to upload", cloud_file_block.len());
        cloud_file_block
    }
    pub async fn select_by_status(&self, status: i8, update_time: DateTime) -> Vec<CloudFileBlock> {
         CloudFileBlock::select_by_status(&self.batis.clone(), status.into(), update_time).await.unwrap()
    }
    pub async fn query_block_need_re_balance(&self, size:i32) -> Vec<TempRow> {
        let rbatis = &self.batis.clone();
        rbatis.query_decode::<Vec<TempRow>>("select file_block_id from (select cfb.file_block_id, count(cfb.id) size from file_block_meta fbm left join cloud_file_block cfb on fbm.id = cfb.file_block_id group by cfb.file_block_id ) where size < ? limit 50", vec![Value::U32(size as u32)])
            .await.unwrap()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TempRow {
    pub file_block_id: Option<i32>,
}
