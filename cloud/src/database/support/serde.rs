use std::{fmt, i8};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::database::meta::{CloudType, FileMetaType};
use crate::database::meta::cloud::MetaStatus;

impl Serialize for MetaStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let value = i8::from(self);
        serializer.serialize_i8(value)
    }
}

impl<'de> Deserialize<'de> for MetaStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct MetaStatusVisitor;

        impl<'de> Visitor<'de> for MetaStatusVisitor {
            type Value = MetaStatus;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("MetaStatus")
            }
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(MetaStatus::from(v as i8))
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(MetaStatus::from(v as i8))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(MetaStatus::from(v as i8))
            }
        }
        deserializer.deserialize_i16(MetaStatusVisitor {})
    }
}

impl Serialize for CloudType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let value = i8::from(self);
        serializer.serialize_i8(value)
    }
}

impl<'de> Deserialize<'de> for CloudType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct CloudTypeVisitor;

        impl<'de> Visitor<'de> for CloudTypeVisitor {
            type Value = CloudType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("CloudType")
            }
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(CloudType::from(v as i8))
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(CloudType::from(v as i8))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(CloudType::from(v as i8))
            }
        }
        deserializer.deserialize_i16(CloudTypeVisitor {})
    }
}

impl Serialize for FileMetaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let value = i32::from(self);
        serializer.serialize_i32(value)
    }
}

impl<'de> Deserialize<'de> for FileMetaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        struct FileMetaTypeVisitor;

        impl<'de> Visitor<'de> for FileMetaTypeVisitor {
            type Value = FileMetaType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("FileMetaType")
            }
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(FileMetaType::from(v as i32))
            }
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(FileMetaType::from(v as i32))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(FileMetaType::from(v as i32))
            }
        }
        deserializer.deserialize_i16(FileMetaTypeVisitor {})
    }
}
