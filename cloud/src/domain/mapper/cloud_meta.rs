use rbatis::{crud, impl_select};

use crate::domain::table::tables::CloudMeta;

crud!(CloudMeta {});
impl_select!(CloudMeta{quota_random(status:i8, size:i32)=>"` where status = #{status} order by remaining_quota * random() desc limit #{size}`"});
