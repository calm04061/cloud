use crate::migrate::version::version_migrate::VersionMigrate;
use crate::migrate::{Migrate, Version};
use api::ResponseResult;
use futures_util::FutureExt;
use sqlx::Transaction;
use sqlx_sqlite::Sqlite;
use std::future::Future;
use std::pin::Pin;

pub(crate) struct DefaultVersionMigrate {
    version: Version,
    sql: String,
}
impl DefaultVersionMigrate {
    pub(crate) fn new(version: &str, sql: &str) -> Self {
        DefaultVersionMigrate {
            version: version.parse::<Version>().unwrap(),
            sql: String::from(sql),
        }
    }
}
impl Migrate for DefaultVersionMigrate {
    fn do_migrate<'a>(&'a mut self, transaction: &'a mut Transaction<'_, Sqlite>) -> Pin<Box<dyn Future<Output=ResponseResult<()>> + Send + 'a>> {
        async move {
            let binding = self.sql.clone();
            let all_sql = binding.as_str();
            self.do_exec(transaction, all_sql).await.expect("TODO: panic message");
            Ok(())
        }.boxed()
    }
}

impl VersionMigrate for DefaultVersionMigrate {
    fn version(&self) -> Version {
        self.version.clone()
    }
}