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
