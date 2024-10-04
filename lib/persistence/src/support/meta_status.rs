use std::fmt;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::MetaStatus;
use crate::MetaStatus::{Disabled, Enable, InvalidRefresh, WaitDataRoot, WaitInit};

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


impl From<i8> for MetaStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => WaitInit,
            1 => WaitDataRoot,
            2 => Enable,
            3 => InvalidRefresh,
            4 => Disabled,
            _ => Disabled,
        }
    }
}


impl From<&MetaStatus> for i8 {
    fn from(value: &MetaStatus) -> Self {
        match value {
            WaitInit => 0,
            WaitDataRoot => 1,
            Enable => 2,
            InvalidRefresh => 3,
            Disabled => 4,
        }
    }
}

impl From<MetaStatus> for i8 {
    fn from(value: MetaStatus) -> Self {
        match value {
            WaitInit => 0,
            WaitDataRoot => 1,
            Enable => 2,
            InvalidRefresh => 3,
            Disabled => 4,
        }
    }
}
