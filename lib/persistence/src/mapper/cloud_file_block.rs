use rbatis::{crud, Error, impl_select, impl_update, RBatis, sql};
use rbatis::rbdc::datetime::DateTime;

use crate::CloudFileBlock;

crud!(CloudFileBlock {});
impl_update!(CloudFileBlock{update_by_status(id: i32,status :i8) =>"` where id=#{id} and status=#{status}`"});

impl CloudFileBlock {
    #[sql("select cfb.* from cloud_file_block cfb left join file_block_meta fbm on cfb.file_block_id = fbm.id and cfb.cloud_file_hash = fbm.part_hash where cfb.status = 1 or (fbm.id is null and cfb.status = 3) order by RANDOM() limit 16")]
    pub(crate) async fn select_to_upload(rb: &mut RBatis) -> Result<Vec<CloudFileBlock>, Error> {
        impled!()
    }
}
impl_select!(CloudFileBlock{
     select_by_file_block_id(file_block_id: i32) => "` where deleted = 0 and file_block_id = #{file_block_id} `"
});

impl_select!(CloudFileBlock{
    select_by_status( status: i8, update_time: DateTime) =>
           "` where status=#{status} and deleted=0 and update_time < #{update_time} `"
});
