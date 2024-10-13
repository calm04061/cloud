use std::cmp::Ordering;
use std::fmt::Display;
use crate::DbPool;
use api::error::ErrorInfo;
use api::ResponseResult;
use sqlx::{Error, Transaction};
use sqlx_sqlite::Sqlite;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

pub(crate) mod version;
pub(crate) mod versions_migrate;
#[derive(Clone)]
pub(crate) struct Version {
    major_version: u16,
    minor_version: u16,
    patch_version: u16,
}
#[derive(Debug)]
pub enum VersionParseError {
    MajorVersionParseError,
    MinorVersionParseError,
    PatchVersionParseError,
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}.{}.{}", self.major_version, self.minor_version, self.patch_version))
    }
}
impl Eq for Version {}

impl PartialEq<Self> for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.major_version > other.major_version {
            return Some(Ordering::Greater);
        }
        if self.major_version < other.major_version {
            return Some(Ordering::Less);
        }
        if self.minor_version > other.minor_version {
            return Some(Ordering::Greater);
        }
        if self.minor_version < other.minor_version {
            return Some(Ordering::Less);
        }
        if self.patch_version > other.patch_version {
            return Some(Ordering::Greater);
        }
        if self.patch_version < other.patch_version {
            return Some(Ordering::Less);
        }
        Some(Ordering::Equal)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major_version > other.major_version {
            return Ordering::Greater;
        }
        if self.major_version < other.major_version {
            return Ordering::Less;
        }
        if self.minor_version > other.minor_version {
            return Ordering::Greater;
        }
        if self.minor_version < other.minor_version {
            return Ordering::Less;
        }
        if self.patch_version > other.patch_version {
            return Ordering::Greater;
        }
        if self.patch_version < other.patch_version {
            return Ordering::Less;
        }
        Ordering::Equal
    }
}
impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('.');
        let mut version = Version {
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
        };
        let mut enumerate = split.enumerate();
        if let Some((_i, ver)) = enumerate.next() {
            match u16::from_str(ver) {
                Ok(v) => version.major_version = v,
                Err(_) => return Err(VersionParseError::MajorVersionParseError),
            }
        }
        if let Some((_i, ver)) = enumerate.next() {
            match u16::from_str(ver) {
                Ok(v) => version.minor_version = v,
                Err(_) => return Err(VersionParseError::MinorVersionParseError),
            }
        }
        if let Some((_i, ver)) = enumerate.next() {
            match u16::from_str(ver) {
                Ok(v) => version.patch_version = v,
                Err(_) => return Err(VersionParseError::PatchVersionParseError),
            }
        }
        Ok(version)
    }
}

pub(crate) trait Migrate {
    fn do_migrate<'a>(&'a mut self, transaction: &'a mut Transaction<'_, Sqlite>) -> Pin<Box<dyn Future<Output=ResponseResult<()>> + Send + 'a>>;
}

pub(crate) async fn query_version(db_pool: DbPool) -> ResponseResult<String> {
    let result: Result<(String,), Error> = sqlx::query_as("select version from migrate order by id desc limit 1")
        .fetch_one(&db_pool)
        .await;
    if let Err(e) = result {
        match e {
            Error::Database(e) => {
                let msg = e.message();
                if msg.starts_with("no such table") {
                    return Err(ErrorInfo::NoSuchTable("migrate".to_string()));
                }
            }
            _ => {}
        }
        return Err(ErrorInfo::OTHER(1, "".to_string()));
    }
    let temp = result?;
    Ok(temp.0)
}