use rbatis::{crud, impl_select};

use crate::meta::CloudMeta;

crud!(CloudMeta {});
impl_select!(CloudMeta{quota_random(status:i8, size:i32)=>"` where status = #{status} and deleted = 0 order by remaining_quota * random() desc limit #{size}`"});
impl_select!(CloudMeta{query_token_timeout(status:i8,now:u64)=>"` where status = #{status} and deleted = 0 and expires_in < #{now}`"});
