use crate::database::meta::cloud::MetaStatus;
use crate::database::meta::{CloudMetaManager, FileManager, FileStatus};
use crate::domain::table::tables::CloudFileBlock;
use crate::pool;
use crate::service::CONTEXT;
use crate::storage::storage_facade::StorageFacade;
use log::{debug, error, info};
use quartz_sched::SchedulerHandle;
use std::fs::File;
use std::io::{ErrorKind, Read};
use rbatis::rbdc::datetime::DateTime;

pub(crate) struct Scan {
    cache_file: String,
}

impl Scan {
    pub(crate) fn new(cache_file: &str) -> Self {
        Scan {
            cache_file: String::from(cache_file),
        }
    }
    async fn do_execute(&self) {
        let cloud_file_block = CloudFileBlock::select_to_upload(pool!()).await.unwrap();
        for mut file_block in cloud_file_block {
            let origin_status = file_block.status;
            file_block.update_time= DateTime::now();
            file_block.status = FileStatus::Uploading.into();
            let count = CloudFileBlock::update_by_status(pool!(), &file_block, file_block.id.unwrap(), origin_status)
                .await.unwrap().rows_affected;
            if count == 0 {
                continue;
            }
            let cloud_meta = CONTEXT
                .cloud_meta_manager
                .info(file_block.cloud_meta_id)
                .await;
            if let None = cloud_meta {
                error!(" storage meta not found");
                return;
            }
            let cloud_meta = cloud_meta.unwrap();
            let status: MetaStatus = cloud_meta.status.into();
            if status != MetaStatus::Enable {
                info!("storage status error {:?}", status);
                return;
            }
            let mut file_block_meta = CONTEXT
                .file_manager
                .file_block_meta_info_by_id(file_block.file_block_id)
                .await
                .unwrap();
            if file_block_meta.part_hash == None {
                error!("{} part_hash is none", file_block_meta.id.unwrap());
                continue;
            }
            let cache_file = format!("{}/{}", self.cache_file, file_block_meta.file_part_id);
            debug!("upload {}", cache_file);
            let result = File::open(cache_file.clone());
            if let Err(e) = result {
                let kink = e.kind();
                match kink {
                    ErrorKind::NotFound => {
                        file_block_meta.status = FileStatus::FileNotExist.into();
                        error!("scan文件读取失败{}", e.kind());
                    }
                    _ => {
                        file_block_meta.status = FileStatus::FileReadError.into();
                        error!("scan文件读取失败{}:{}", e.kind(), e.to_string());
                    }
                }
                let result = CONTEXT
                    .file_manager
                    .update_file_block_meta(file_block_meta)
                    .await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        error!("更新文件块出错{}", e);
                    }
                }
                continue;
            }
            let mut file = result.unwrap();
            let mut body: Vec<u8> = Vec::new();
            let result = file.read_to_end(&mut body);
            if let Err(e) = result {
                file_block_meta.status = FileStatus::FileReadError.into();
                error!("文件读取失败{}:{}", e.kind(), e.to_string());
                let result = CONTEXT
                    .file_manager
                    .update_file_block_meta(file_block_meta)
                    .await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        error!("更新文件块出错{}", e);
                    }
                }
                continue;
            }

            let mut cloud = StorageFacade::new();
            let result = cloud
                .upload_content(&file_block_meta, &cloud_meta, &body)
                .await;
            match result {
                Ok(cr) => {
                    file_block.status = FileStatus::UploadSuccess.into();
                    file_block.cloud_file_id = Some(cr.file_id);
                    file_block.update_time= DateTime::now();
                    file_block.cloud_file_hash = file_block_meta.part_hash;
                    CloudFileBlock::update_by_status(
                        pool!(),
                        &file_block,
                        file_block.id.unwrap(),
                        FileStatus::Uploading.into(),
                    )
                        .await
                        .unwrap();
                    debug!("upload {} done", cache_file);
                }
                Err(e) => {
                    file_block.status = FileStatus::UploadFail.into();
                    file_block.update_time= DateTime::now();
                    error!("上传文件{}:{}", cache_file, e.to_string());
                    CloudFileBlock::update_by_status(
                        pool!(),
                        &file_block,
                        file_block.id.unwrap(),
                        FileStatus::Uploading.into(),
                    )
                        .await
                        .unwrap();
                }
            }
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
