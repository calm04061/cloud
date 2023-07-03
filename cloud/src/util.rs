// use std::path::Path;
//
// use log::{error, info};

// pub fn mk_dir(path_str: &str) {
//     let path = Path::new(path_str);
//     if path.exists() {
//         return;
//     }
//     let result = std::fs::create_dir_all(path_str);
//     match result {
//         Ok(_) => {
//             info!("{}创建完成", path_str)
//         }
//         Err(e) => {
//             error!("{:?}创建失败", e)
//         }
//     }
// }
use std::collections::{HashMap, VecDeque};

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