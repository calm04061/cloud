use crate::migrate::{Migrate, Version};
use api::ResponseResult;
use futures_util::FutureExt;
use sqlx::{Executor, Transaction};
use sqlx_sqlite::Sqlite;
use std::future::Future;
use std::pin::Pin;
pub(crate) trait VersionMigrate: Migrate
{
    fn version(&self) -> Version;
    fn do_exec<'a>(&'a mut self, transaction: &'a mut Transaction<'_, Sqlite>, all_sql: &'a str) -> Pin<Box<dyn Future<Output=ResponseResult<()>> + Send + 'a>> {
        let version_str = self.version().to_string();
        async move{
            let split = all_sql.split(";");
            for sql in split {
                let temp_sql = sql.trim();
                if temp_sql.is_empty() {
                    continue;
                }
                let result = transaction.execute(temp_sql)
                    .await;
                if let Err(e) = result {
                    return Err(e.into());
                }
            }
            let string = format!("insert into migrate (version) values ('{}')", version_str);
            transaction.execute(string.as_str())
                .await?;
            Ok(())
        }.boxed()
    }
}
