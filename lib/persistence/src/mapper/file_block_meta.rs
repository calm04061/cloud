use rbatis::{crud, impl_select, impl_update};

use crate::meta::FileBlockMeta;

crud!(FileBlockMeta {});
impl_update!(FileBlockMeta{update_by_status(id: i32,status :i8) =>"` where id=#{id} and status=#{status}`"});
impl_update!(FileBlockMeta{
    update_by_file_meta_id_and_block_index(file_meta_id:u64, block_index:i64) =>
           "` where deleted = 0 and file_meta_id = #{file_meta_id} and block_index = #{block_index} `"
});
impl_select!(FileBlockMeta{
    select_by_file_meta_id_and_block_index(file_meta_id:u64, block_index:i64) =>
           "` where deleted = 0 and file_meta_id = #{file_meta_id} and block_index = #{block_index} `"
});
impl_select!(FileBlockMeta{
     select_by_file_meta_id(file_meta_id: u64) => "` where deleted = 0 and file_meta_id = #{file_meta_id} `"
});
impl_select!(FileBlockMeta{
     select_by_status_limit(status: i8, size: usize) => "` where status=#{status} limit #{size} `"
});
