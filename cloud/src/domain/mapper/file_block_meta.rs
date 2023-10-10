use rbatis::{crud, impl_select, impl_update};

use crate::domain::table::tables::FileBlockMeta;

crud!(FileBlockMeta {});

impl_update!(FileBlockMeta{
    update_by_file_meta_id_and_block_index(file_meta_id:i32, block_index:i64) =>
           "` where deleted = 0 and file_meta_id = #{file_meta_id} and block_index = #{block_index} `"
});
impl_select!(FileBlockMeta{
    select_by_file_meta_id_and_block_index(file_meta_id:i32, block_index:i64) =>
           "` where deleted = 0 and file_meta_id = #{file_meta_id} and block_index = #{block_index} `"
});
impl_select!(FileBlockMeta{
     select_by_file_meta_id(file_meta_id: i32) => "` where deleted = 0 and file_meta_id = #{file_meta_id} `"
});
