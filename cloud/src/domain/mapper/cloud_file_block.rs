use crate::domain::table::tables::CloudFileBlock;
use rbatis::{crud, impl_update, Error, Rbatis};
crud!(CloudFileBlock {});
impl_update!(CloudFileBlock{update_by_status(id: i64,status :i8) =>"` where id=#{id} and status=#{status}`"});

impl CloudFileBlock {
    #[sql("select cfb.* from cloud_file_block cfb join file_block_meta fbm on cfb.file_block_id = fbm.id where ((cfb.cloud_file_hash != fbm.part_hash) or cfb.cloud_file_hash is null)")]
    pub(crate) async fn select_to_upload(rb: &mut Rbatis) -> Result<Vec<CloudFileBlock>, Error> {
        impled!()
    }
}
impl_select!(CloudFileBlock{
     select_by_file_block_id(file_block_id: i64) => "` where deleted = 0 and file_block_id = #{file_block_id} `"
});
