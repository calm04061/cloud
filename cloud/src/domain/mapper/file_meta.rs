use crate::domain::table::tables::FileMeta;
use rbatis::{crud, impl_select};
crud!(FileMeta {});
impl_select!(FileMeta{select_by_parent(parent_id:i32)=>"` where parent_id = #{parent_id} and deleted = 0`"});
impl_select!(FileMeta{info_by_parent_and_name(parent_id:i32, name:&str)=>"` where parent_id=#{parent_id} and name = #{name} and deleted = 0`"});
impl_select!(FileMeta{list_deleted_file(update_time:i64)=>"` where update_time <= #{update_time} and deleted = 1 and status <> 7 order by update_time desc limit 1000`"});
