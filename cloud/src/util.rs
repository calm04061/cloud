use std::collections::{HashMap, VecDeque};
use quickxml_to_serde::{Config, xml_string_to_json};
use serde::Deserialize;

use serde_json::Error;

use crate::error::ErrorInfo;

pub trait IntoOne<V> {
    fn into_one(self) -> Option<V>;
}

impl<V> IntoOne<V> for Option<V> {
    fn into_one(self) -> Option<V> {
        self
    }
}

impl<V> IntoOne<V> for Vec<V> {
    fn into_one(self) -> Option<V> {
        self.into_iter().next()
    }
}

impl<V> IntoOne<V> for VecDeque<V> {
    fn into_one(self) -> Option<V> {
        self.into_iter().next()
    }
}

impl<K, V> IntoOne<(K, V)> for HashMap<K, V> {
    fn into_one(self) -> Option<(K, V)> {
        self.into_iter().next()
    }
}

impl From<Error> for ErrorInfo {
    fn from(value: Error) -> Self {
        ErrorInfo::OTHER(102, value.to_string())
    }
}
impl From<reqwest::Error> for ErrorInfo {
    fn from(value: reqwest::Error) -> Self {
        ErrorInfo::OTHER(20, value.to_string())
    }
}

pub(crate) trait ToXml {
    fn to_xml(&self, buf: &mut String);
    fn to_xml_with_header(&self, buf: &mut String) {
        buf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        self.to_xml(buf);
    }
}

pub(crate) fn from_xml<T>(xml: String, root: &str) -> Result<T, serde_json::Error>
    where T: for<'de> Deserialize<'de>
{
    let json = xml_string_to_json(xml, &Config::new_with_defaults());
    let value = json.unwrap();
    let option = value.get(root).unwrap();
    serde_json::from_value(option.clone())
}

pub(crate) fn from_xml_default<T>(xml: String) -> Result<T, serde_json::Error>
    where T: for<'de> Deserialize<'de> {
    from_xml(xml,"result")
}