use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::FileMetaType;
use crate::FileMetaType::{DIR, FILE, SYMLINK};

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

impl FileMetaType {
    pub fn is_file(code: i8) -> bool {
        return code == FILE.get_code();
    }
    pub fn is_dir(code: i8) -> bool {
        return code == DIR.get_code();
    }
    pub fn get_code(&self) -> i8 {
        return if self == &FILE { 1 } else { 2 };
    }
}

impl From<i8> for FileMetaType {
    fn from(value: i8) -> Self {
        match value {
            1 => FILE,
            2 => DIR,
            _ => FILE,
        }
    }
}

impl From<i32> for FileMetaType {
    fn from(value: i32) -> Self {
        match value {
            1 => FILE,
            2 => DIR,
            3 => SYMLINK,
            _ => FILE
        }
    }
}

impl From<&FileMetaType> for i32 {
    fn from(value: &FileMetaType) -> Self {
        match value {
            FILE => 1,
            DIR => 2,
            SYMLINK => 3
        }
    }
}
