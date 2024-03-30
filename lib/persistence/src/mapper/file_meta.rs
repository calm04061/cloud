use rbatis::{crud, impl_select};

use crate::meta::FileMeta;

crud!(FileMeta {});
impl_select!(FileMeta{select_by_parent(parent_id:u64)=>"` where parent_id = #{parent_id} and deleted = 0`"});
impl_select!(FileMeta{select_by_parent_page(parent_id:u64, start:u64, size:usize)=>"` where parent_id = #{parent_id} and id> #{start} and deleted = 0 order by id  limit #{size}`"});
impl_select!(FileMeta{info_by_parent_and_name(parent_id:u64, name:&str)=>"` where parent_id=#{parent_id} and name = #{name} and deleted = 0`"});
impl_select!(FileMeta{list_deleted_file(update_time:i64)=>"` where update_time <= #{update_time} and file_type in (1,3) and deleted = 1 and status <> 7 order by update_time desc limit 1000`"});
