// use r2d2_sqlite::rusqlite::{Error, ToSql};
// use r2d2_sqlite::rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, Value, ValueRef};
// use r2d2_sqlite::rusqlite::types::ToSqlOutput::Owned;
//
// use crate::database::meta::{CloudType, FileMetaType};
// use crate::database::meta::storage::MetaStatus;
// use crate::database::meta::storage::MetaStatus::Disabled;
// use crate::database::meta::CloudType::AliYun;
// use crate::database::meta::FileMetaType::FILE;
// use crate::error::ErrorInfo;
//
// impl FromSql for MetaStatus {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         let result = value.as_i64_or_null();
//         if let Err(e) = result {
//             return Err(e);
//         }
//         let option = result.unwrap();
//         let status = match option {
//             None => Disabled,
//             Some(v) => MetaStatus::from(v as i8),
//         };
//         Ok(status)
//     }
// }
//
// impl ToSql for MetaStatus {
//     fn to_sql(&self) -> r2d2_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
//         let i = i8::from(self);
//         Ok(Owned(Value::from(i)))
//     }
// }
//
// impl FromSql for CloudType {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         let result = value.as_i64_or_null();
//         if let Err(e) = result {
//             return Err(e);
//         }
//         let option = result.unwrap();
//         let status = match option {
//             None => AliYun,
//             Some(v) => CloudType::from(v as i8),
//         };
//         Ok(status)
//     }
// }
//
// impl ToSql for FileMetaType {
//     fn to_sql(&self) -> r2d2_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
//         let i = i32::from(self);
//         Ok(Owned(Value::from(i)))
//     }
// }
// impl FromSql for FileMetaType {
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         let result = value.as_i64_or_null();
//         if let Err(e) = result {
//             return Err(e);
//         }
//         let option = result.unwrap();
//         let status = match option {
//             None => FILE,
//             Some(v) => FileMetaType::from(v as i32),
//         };
//         Ok(status)
//     }
// }
//
// impl ToSql for CloudType {
//     fn to_sql(&self) -> r2d2_sqlite::rusqlite::Result<ToSqlOutput<'_>> {
//         let i = i8::from(self);
//         Ok(Owned(Value::from(i)))
//     }
// }
//
// impl  From<Error> for ErrorInfo {
//     fn from(value: Error) -> Self {
//         ErrorInfo::new_string(4,format!("{}",value))
//     }
// }
// impl  From<r2d2::Error> for ErrorInfo {
//     fn from(value: r2d2::Error) -> Self {
//         ErrorInfo::new_string(4,format!("{}",value))
//     }
// }
