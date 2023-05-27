use crate::domain::table::tables::CloudMeta;
use rbatis::{crud, impl_select};
crud!(CloudMeta {});
impl_select!(CloudMeta{quota_random(status:i8, size:i32)=>"` where status = #{status} order by remaining_quota * random() desc limit #{size}`"});