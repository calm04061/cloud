use crate::domain::table::tables::CloudFileBlock;
use rbatis::{crud, Error, impl_update, RBatis};
use rbatis::rbdc::datetime::DateTime;
crud!(CloudFileBlock {});
impl_update!(CloudFileBlock{update_by_status(id: i64,status :i8) =>"` where id=#{id} and status=#{status}`"});

impl CloudFileBlock {
    #[sql("select cfb.* from cloud_file_block cfb join file_block_meta fbm on cfb.file_block_id = fbm.id where ((cfb.cloud_file_hash != fbm.part_hash) or cfb.cloud_file_hash is null) and cfb.status<>2 and cfb.status<>7 ")]
    pub(crate) async fn select_to_upload(rb: &mut RBatis) -> Result<Vec<CloudFileBlock>, Error> {
        impled!()
    }
}
impl_select!(CloudFileBlock{
     select_by_file_block_id(file_block_id: i64) => "` where deleted = 0 and file_block_id = #{file_block_id} `"
});

impl_select!(CloudFileBlock{
    select_by_status( status: i8, update_time: DateTime) =>
           "` where status=#{status} and deleted=0 and update_time < #{update_time} `"
});
