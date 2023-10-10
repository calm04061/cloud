use std::fs::File;
use std::io::{ErrorKind, Read};

use log::{debug, info};
use quartz_sched::SchedulerHandle;
use rbatis::rbdc::datetime::DateTime;

use crate::database::meta::{CloudMetaManager, EventType, FileManager, FileStatus};
use crate::database::meta::cloud::MetaStatus;
use crate::domain::table::tables::{CloudFileBlock, EventMessage};
use crate::error::ErrorInfo;
use crate::pool;
use crate::service::CONTEXT;
use crate::storage::storage::ResponseResult;
use crate::storage::storage_facade::StorageFacade;

pub(crate) struct Scan {
    cache_file: String,
}

impl Scan {
    pub(crate) fn new(cache_file: &str) -> Self {
        Scan {
            cache_file: String::from(cache_file),
        }
    }
    async fn do_execute_one_block(&self, file_block: &mut CloudFileBlock) -> ResponseResult<()> {
        let cloud_meta = CONTEXT
            .cloud_meta_manager
            .info(file_block.cloud_meta_id)
            .await;
        if let None = cloud_meta {
            return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] not found", file_block.cloud_meta_id)));
        }
        let cloud_meta = cloud_meta.unwrap();
        let status: MetaStatus = cloud_meta.status.into();
        if status != MetaStatus::Enable {
            info!("storage status error {:?}", status);
            return Err(ErrorInfo::OTHER(0, format!(" storage meta [{}] status error {:?}", file_block.cloud_meta_id, status)));
        }
        let mut file_block_meta = CONTEXT
            .file_manager
            .file_block_meta_info_by_id(file_block.file_block_id)
            .await
            .unwrap();

        let cache_file = format!("{}/{}", self.cache_file, file_block_meta.file_part_id);
        debug!("upload {}", cache_file);
        let result = File::open(cache_file.clone());
        if let Err(e) = result {
            let kink = e.kind();
            let info = match kink {
                ErrorKind::NotFound => {
                    file_block_meta.status = FileStatus::FileNotExist.into();
                    ErrorInfo::OTHER(2, format!("scan文件读取失败{}", e.kind()))
                }
                _ => {
                    file_block_meta.status = FileStatus::FileReadError.into();
                    ErrorInfo::OTHER(2, format!("scan文件读取失败{}:{}", e.kind(), e.to_string()))
                }
            };
            CONTEXT
                .file_manager
                .update_file_block_meta(file_block_meta)
                .await?;
            return Err(info);
        }
        let mut file = result.unwrap();
        let mut body: Vec<u8> = Vec::new();
        let result = file.read_to_end(&mut body);
        if let Err(e) = result {
            file_block_meta.status = FileStatus::FileReadError.into();
            let result = CONTEXT
                .file_manager
                .update_file_block_meta(file_block_meta)
                .await;
            return match result {
                Ok(_) => {
                    Err(ErrorInfo::OTHER(2, format!("文件读取失败{}:{}", e.kind(), e.to_string())))
                }
                Err(e) => {
                    Err(e)
                }
            }
        }

        let mut cloud = StorageFacade::new();
        let result = cloud
            .upload_content(&file_block_meta, &cloud_meta, &body)
            .await;
        return match result {
            Ok(cr) => {
                file_block.status = FileStatus::UploadSuccess.into();
                file_block.cloud_file_id = Some(cr.file_id);
                file_block.update_time = DateTime::now();
                file_block.cloud_file_hash = Some(file_block_meta.part_hash);
                CloudFileBlock::update_by_status(
                    pool!(),
                    &file_block,
                    file_block.id.unwrap(),
                    FileStatus::Uploading.into(),
                )
                    .await?;
                Ok(())
            }
            Err(e) => {
                file_block.status = FileStatus::UploadFail.into();
                file_block.update_time = DateTime::now();
                CloudFileBlock::update_by_status(
                    pool!(),
                    &file_block,
                    file_block.id.unwrap(),
                    FileStatus::Uploading.into(),
                )
                    .await?;
                Err(ErrorInfo::OTHER(0, format!("上传文件{}:{}", cache_file, e.to_string())))
            }
        };
    }
    async fn do_execute(&self) {
        let cloud_file_block = CloudFileBlock::select_to_upload(pool!()).await.unwrap();
        for mut file_block in cloud_file_block {
            let origin_status = file_block.status;
            file_block.update_time = DateTime::now();
            file_block.status = FileStatus::Uploading.into();
            let count = CloudFileBlock::update_by_status(pool!(), &file_block, file_block.id.unwrap(), origin_status)
                .await.unwrap().rows_affected;
            if count == 0 {
                continue;
            }
            let result = self.do_execute_one_block(&mut file_block).await;
           let message= match result {
                Ok(_) => {
                    EventMessage::success(EventType::UploadFileBlock, format!("{} upload success", file_block.id.unwrap()))
                }
                Err(e) => {
                    EventMessage::fail(EventType::UploadFileBlock, format!("{} upload fail:{}", file_block.id.unwrap(), e.to_string()))
                }
            };
            EventMessage::insert(pool!(),&message).await.unwrap();
        }
    }
}

impl quartz_sched::Job for Box<Scan> {
    fn execute(&self, _engine: Option<SchedulerHandle>) {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.do_execute());
    }

    fn description(&self) -> String {
        String::from("Scan")
    }

    fn key(&self) -> i64 {
        1
    }
}
