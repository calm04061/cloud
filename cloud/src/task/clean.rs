use crate::database::meta::{FileManager, FileStatus};
use crate::domain::table::tables::{CloudFileBlock, FileBlockMeta, FileMeta};
use crate::error::ErrorInfo;
use crate::pool;
use crate::service::CONTEXT;
use crate::storage::storage_facade::StorageFacade;
use log::{debug, error, info};
use quartz_sched::SchedulerHandle;

pub(crate) struct Clean {
    cache_file: String,
}

impl Clean {
    pub(crate) fn new(cache_file: &str) -> Self {
        Clean {
            cache_file: String::from(cache_file),
        }
    }
}

impl quartz_sched::Job for Box<Clean> {
    fn execute(&self, _engine: Option<SchedulerHandle>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let mut cloud = StorageFacade::new();

                let ten = chrono::Local::now().timestamp_millis() - 10 * 1000;
                let file_metas = CONTEXT.file_manager.list_deleted_file(ten).await;

                for mut file_meta in file_metas {
                    clean_file_block(&mut file_meta, &mut cloud, &self.cache_file).await;
                }
            });
    }

    fn description(&self) -> String {
        String::from("清理")
    }

    fn key(&self) -> i64 {
        2
    }
}

async fn clean_file_block(file_meta: &mut FileMeta, cloud: &mut StorageFacade, cache_file: &str) {
    let file_block_metas = CONTEXT
        .file_manager
        .file_block_meta(file_meta.id.unwrap())
        .await;
    if file_block_metas.is_empty() {
        file_meta.status = FileStatus::WaitClean.into();
        FileMeta::update_by_column(pool!(), file_meta, "id")
            .await
            .ok();
        return;
    }
    for mut file_block_meta in file_block_metas {
        let size = delete_cloud_file_block(file_meta, &file_block_meta, cloud).await;
        if size == 0 {
            let local_cache_file = format!("{}/{}", cache_file, file_block_meta.file_part_id);
            let result = std::fs::remove_file(local_cache_file.clone());
            if let Err(e) = result {
                info!("clean->{},{}:{}", file_meta.name, local_cache_file, e)
            }
            file_block_meta.deleted = 1;
            FileBlockMeta::update_by_column(pool!(), &file_block_meta, "id")
                .await
                .unwrap();
            continue;
        }
    }
}

async fn delete_cloud_file_block(file_meta: &mut FileMeta, file_block_meta: &FileBlockMeta, cloud: &mut StorageFacade) -> usize {
    let cloud_file_blocks = CloudFileBlock::select_by_file_block_id(pool!(), file_block_meta.id.unwrap())
            .await
            .unwrap();
    let size = cloud_file_blocks.len();

    for mut cloud_file_block in cloud_file_blocks {
        let result = cloud.delete(&cloud_file_block).await;
        if let Err(e) = result {
            match e {
                ErrorInfo::FileNotFound(_) => {
                    cloud_file_block.deleted = 1;
                    CloudFileBlock::update_by_column(pool!(), &cloud_file_block, "id")
                        .await
                        .unwrap();
                }
                ErrorInfo::NoneCloudFileId(_) => {
                    cloud_file_block.deleted = 1;
                    CloudFileBlock::update_by_column(pool!(), &cloud_file_block, "id")
                        .await
                        .unwrap();
                }
                _ => {
                    error!("删除云文件失败:{},{}", file_meta.name, e);
                }
            }
        } else {
            debug!("删除云文件成功");
            cloud_file_block.deleted = 1;
            CloudFileBlock::update_by_column(pool!(), &cloud_file_block, "id")
                .await
                .unwrap();
        }
    }
    size
}

