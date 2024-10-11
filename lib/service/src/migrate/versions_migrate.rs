use std::future::{Future};
use std::pin::Pin;
use futures_util::FutureExt;
use std::sync::{Arc};
use sqlx::Transaction;
use sqlx_sqlite::Sqlite;
use tokio::sync::Mutex;
use api::ResponseResult;
use crate::migrate::Version;
use crate::migrate::version::version_migrate::VersionMigrate;

pub(crate) struct VersionsMigrate {
    versions: Arc<Mutex<Vec<Box<dyn VersionMigrate + Send>>>>,
    pre_version: Version,
}

impl VersionsMigrate {
    pub(crate) fn new(pre_version: Version) -> Self {
        let versions = VersionsMigrate {
            versions: Arc::new(Default::default()),
            pre_version,
        };
        versions
    }

    pub(crate) async fn add(&mut self, version_migrate: Box<dyn VersionMigrate + Send>) {
        let mut versions = self.versions.lock().await;
        if version_migrate.version().gt(&self.pre_version) {
            versions.push(version_migrate);
        }
        versions.sort_by(|a, b| {
            // Sort by version number
            a.version().cmp(&b.version())
        });
    }

    pub(crate) fn do_migrate<'a>(&'a mut self, transaction: &'a mut Transaction<'_, Sqlite>) -> Pin<Box<dyn Future<Output=ResponseResult<()>> + Send + 'a>> {
        async move {
            // Lock the mutex to get a copy of the versions vector
            let mut versions = self.versions.lock().await;
            for version in &mut *versions {
                version.do_migrate(transaction).await.expect("TODO: panic message");
            }
            Ok(())
        }.boxed()
    }
}
