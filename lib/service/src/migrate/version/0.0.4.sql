-- 0.0.4: 上传队列重试治理（#15）
-- retry_count: 连续失败次数，达到上限后进入死信（status=10 Dead）
-- next_retry_time: 退避后的下次可重试时间，扫描只处理已到期的行
alter table cloud_file_block add column retry_count integer not null default 0;
alter table cloud_file_block add column next_retry_time datetime;
