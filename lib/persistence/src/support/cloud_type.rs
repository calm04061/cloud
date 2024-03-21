use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::CloudType;

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

impl From<i8> for CloudType {
    fn from(value: i8) -> Self {
        match value {
            1 => CloudType::AliYun,
            2 => CloudType::Baidu,
            3 => CloudType::Local,
            4 => CloudType::OneDrive,
            #[cfg(not(windows))]
            5 => CloudType::Sftp,
            _ => CloudType::AliYun,
        }
    }
}

impl From<&CloudType> for i8 {
    fn from(value: &CloudType) -> Self {
        match value {
            CloudType::AliYun => 1,
            CloudType::Baidu => 2,
            CloudType::Local => 3,
            CloudType::OneDrive => 4,
            #[cfg(not(windows))]
            CloudType::Sftp => 5,
        }
    }
}
impl From<&CloudType> for String {
    fn from(value: &CloudType) -> Self {
        match value {
            CloudType::AliYun => "阿里云盘".to_string(),
            CloudType::Baidu => "百度云盘".to_string(),
            CloudType::Local => "本地磁盘".to_string(),
            CloudType::OneDrive => "OneDrive".to_string(),
            #[cfg(not(windows))]
            CloudType::Sftp => "Sftp".to_string(),
        }
    }
}

impl From<CloudType> for i8 {
    fn from(value: CloudType) -> Self {
        return i8::from(&value);
    }
}