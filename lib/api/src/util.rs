use std::collections::{HashMap, VecDeque};

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