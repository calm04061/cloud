use std::time::Duration;

use bytes::Bytes;
use log::{debug, info};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use task_local_extensions::Extensions;
use urlencoding::encode;

use crate::domain::table::tables::CloudMeta;
use crate::error::ErrorInfo;
use crate::error::ErrorInfo::{Http, Http302};
use crate::storage::baidu::baidu_authorization_middleware::{BaiduAuthMiddleware, Token};
use crate::storage::baidu::vo::{
    AsyncType, BaiduCreate, BaiduFileManagerResult, BaiduOpera, BaiduPreCreate, BaiduQuota,
    BaiduUser, FileMetas, Query, Search,
};
use crate::storage::storage::{
    CreateResponse, FileInfo, FileItemWrapper, Network, Quota, ResponseResult, SearchResponse,
    Storage, StorageFile, User,
};
use async_trait::async_trait;
use crypto::digest::Digest;
use crypto::md5::Md5;
use rbatis::utils::into_one::IntoOne;
use reqwest::multipart::{Form, Part};
use serde::Serialize;
use crate::database::meta::cloud::MetaStatus;

// const CHUNK_SIZE: usize = 10485760;
const BLOCK_SIZE: usize = 1024 * 1024 * 4;
pub const API_DOMAIN_PREFIX: &str = "https://pan.baidu.com";
pub const AUTH_DOMAIN_PREFIX: &str = "https://openapi.baidu.com";
pub const FILE_DOMAIN_PREFIX: &str = "https://d.pcs.baidu.com";

struct Inner {
    api_client: ClientWithMiddleware,
    content_client: ClientWithMiddleware,
    user: Option<User>,
}

pub struct BaiduStorage {
    // api_client: ClientWithMiddleware,
    // content_client: ClientWithMiddleware,
    // user: Option<User>,
    inner: Inner,
}

impl BaiduStorage {
    async fn get_drive_id(&mut self, cloud_meta: CloudMeta) -> ResponseResult<String> {
        let option = self.inner.user.clone();
        let user = match option {
            None => self.user_info(cloud_meta).await?,
            Some(u) => u.clone(),
        };
        return Ok(user.default_drive_id.unwrap());
    }
}

impl BaiduStorage {
    pub fn new() -> Self {
        let auth_middleware = BaiduAuthMiddleware::new();
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(auth_middleware).build();

        let content_client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let content_client = ClientBuilder::new(content_client).build();
        BaiduStorage {
            inner: Inner {
                api_client,
                content_client,
                user: None,
            },
        }
    }
}

#[async_trait::async_trait]
impl Storage for BaiduStorage {

    async fn user_info(&mut self, cloud_meta: CloudMeta) -> ResponseResult<User> {
        self.inner.user_info(cloud_meta).await
    }
}

impl Clone for BaiduStorage {
    fn clone(&self) -> Self {
        let auth_middleware = BaiduAuthMiddleware::new();
        let client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let api_client = ClientBuilder::new(client).with(auth_middleware).build();

        let content_client = Client::builder()
            // .proxy(reqwest::Proxy::https("http://127.0.0.1:8888").unwrap())
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(300))
            .build()
            .unwrap();
        let content_client = ClientBuilder::new(content_client).build();
        BaiduStorage {
            inner: Inner {
                api_client,
                content_client,
                user: self.inner.user.clone(),
            },
        }
    }
}

#[async_trait]
impl StorageFile for BaiduStorage {
    // async fn upload(
    //     &mut self,
    //     parent_file_id: &str,
    //     name: &str,
    //     file_path: &str,
    //     cloud_meta: CloudMeta,
    // ) -> ResponseResult<CreateResponse> {
    //     //https://api.aliyundrive.com/adrive/v2/file/createWithFolders
    //
    //     let file = File::open(file_path).unwrap();
    //     let metadata = file.metadata().unwrap();
    //     let len = metadata.len();
    //     // let mut content = BufReader::new(file);
    //
    //     let chunk_count = (len + CHUNK_SIZE as u64 - 1) / CHUNK_SIZE as u64;
    //
    //     let mut part_infos = vec![];
    //     let mut index = 0;
    //     while index < chunk_count {
    //         part_infos.insert(
    //             index as usize,
    //             PartInfo {
    //                 part_number: (index + 1) as u32,
    //                 upload_url: None,
    //                 internal_upload_url: None,
    //                 content_type: None,
    //             },
    //         );
    //         index += 1;
    //     }
    //
    //     let file = File::open(file_path).unwrap();
    //
    //     let content = BufReader::new(file);
    //     let x = content.buffer().to_vec();
    //     self.upload_content(parent_file_id, name, &x, cloud_meta).await
    // }
    async fn upload_content(
        &mut self,
        name: &str,
        content: &Vec<u8>,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let len = content.len();
        let mut blocks = vec![];
        let mut start = 0;
        let mut end = BLOCK_SIZE;
        let mut md5s = vec![];
        while start < len {
            if end > len {
                end = len;
            }
            let block = content[start..end].to_vec();
            let mut md5 = Md5::new();
            md5.input(&block);
            let md5_value = md5.result_str();
            md5s.push(md5_value);
            blocks.push(block);
            start = start + BLOCK_SIZE;
            end = end + BLOCK_SIZE;
        }

        let size = len.to_string();
        let size = size.as_str();

        let block_list = serde_json::to_string(&md5s);
        let block_list = block_list.unwrap();
        let block_list = block_list.as_str();
        let path = format!("{}/{}", cloud_meta.data_root.as_ref().unwrap(), name);
        let mut par = vec![];
        // let mut parameter = HashMap::new();
        par.push(("path", path.as_str().clone()));
        par.push(("size", size));
        par.push(("isdir", "0"));
        par.push(("autoinit", "1"));
        par.push(("rtype", "3"));
        par.push(("block_list", block_list));

        info!("block_list:{}", block_list);
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let result = self
            .inner
            .post_form("rest/2.0/xpan/file?method=precreate", &par, &mut extensions)
            .await;

        let json = result.unwrap();
        info!("precreate:{}", json);
        let result: BaiduPreCreate = serde_json::from_str(json.as_str()).unwrap();
        for (index, block) in blocks.iter_mut().enumerate() {
            let upload_id = result.uploadid.clone();
            let upload_id = upload_id.unwrap();
            let mut extensions = Extensions::new();
            extensions.insert(cloud_meta.clone());
            let x = block.clone();
            let bio = Part::bytes(x)
                .file_name("1")
                .mime_str("application/octet-stream")
                .unwrap();
            // let index = index.to_string();
            // let index = index.clone().as_str();
            let form = Form::new().part("file", bio);
            let requet_query = format!(
                "rest/2.0/pcs/superfile2?method=upload&type=tmpfile&path={}&uploadid={}&partseq={}",
                path.clone(),
                upload_id.clone(),
                index.to_string()
            );
            let resp_result = self
                .inner
                .api_client
                .post(format!("{}/{}", FILE_DOMAIN_PREFIX, requet_query.as_str()))
                .multipart(form)
                .build()
                .unwrap();
            let resp_result = self
                .inner
                .api_client
                .execute_with_extensions(resp_result, &mut extensions);
            let string = self.inner.get_response_text(resp_result).await.unwrap();
            info!("upload:{}", string);
            // self.post_file("rest/2.0/pcs/superfile2?method=upload", form, &mut extensions).await.unwrap();
        }
        let uploadid = result.uploadid.clone();
        let uploadid = uploadid.unwrap();
        let mut vec1 = par.clone();
        vec1.push(("uploadid", uploadid.as_str()));
        info!("start create");

        let result = self
            .inner
            .post_form("rest/2.0/xpan/file?method=create", &vec1, &mut extensions)
            .await;
        let json = result.unwrap();
        info!("create:{}", json);
        let result: BaiduCreate = serde_json::from_str(json.as_str()).unwrap();
        return Ok(CreateResponse {
            domain_id: "".to_string(),
            drive_id: "".to_string(),
            encrypt_mode: "".to_string(),
            file_id: result.fs_id.unwrap().to_string(),
            file_name: "".to_string(),
            location: "".to_string(),
            parent_file_id: "".to_string(),
            rapid_upload: false,
            file_type: "".to_string(),
            upload_id: "".to_string(),
        });
    }

    async fn search(
        &mut self,
        parent_file_id: &str,
        name: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<SearchResponse> {
        let drive_id = self.get_drive_id(cloud_meta).await?;
        let query = format!(
            "parent_file_id = \"{}\" and (name = \"{}\")",
            parent_file_id, name
        );
        let search = Search {
            drive_id,
            limit: 100,
            order_by: "name ASC".to_string(),
            query,
        };
        let resp_result = self
            .inner
            .api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "adrive/v3/file/search"))
            .json(&search)
            .send();
        let json = self.inner.get_response_text(resp_result).await?;
        let result = serde_json::from_str(json.as_str());
        return Ok(result.unwrap());
    }

    async fn delete(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<()> {
        let result = self.info(file_id, cloud_meta.clone()).await;
        if let Err(e) = result {
            return if e == Http(404) { Ok(()) } else { Err(e) };
        }

        let info = result.unwrap();
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let mut file_lists = vec![];
        file_lists.push(info.path.unwrap());
        let result = self
            .inner
            .manage_file(
                BaiduOpera::Delete,
                &file_lists,
                AsyncType::Async,
                &cloud_meta,
            )
            .await;
        match result {
            Ok(str) => {
                debug!("delete file result :{:?}", str);
                return Ok(());
            }
            Err(e) => Err(e),
        }
    }

    async fn content(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<Bytes> {
        debug!("get_download_url");
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let file_info = self.info(file_id, cloud_meta.clone()).await;
        let info = file_info.unwrap();
        let mut download_url = info.download_url.unwrap();
        loop {
            let result = self
                .inner
                .download(download_url.as_str(), &cloud_meta)
                .await;
            if let Ok(bytes) = result {
                return Ok(bytes);
            }
            let error_code = result.err().unwrap();
            if let Http302(url) = error_code {
                download_url = url;
            } else {
                return Err(error_code);
            }
        }
    }

    async fn info(&mut self, file_id: &str, cloud_meta: CloudMeta) -> ResponseResult<FileInfo> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let mut fsids = vec![];
        fsids.push(file_id.parse::<i64>().unwrap());
        let fsids = serde_json::to_string(&fsids).unwrap();
        let result = self
            .inner
            .get_json(
                format!(
                    "rest/2.0/xpan/multimedia?method=filemetas&dlink=1&fsids={}",
                    fsids
                )
                    .as_str(),
                &mut extensions,
            )
            .await
            .unwrap();

        let result: FileMetas = serde_json::from_str(result.as_str()).unwrap();
        let file_metas = result.list;
        if file_metas.is_empty() {
            return Err(ErrorInfo::FileNotFound(format!("{}不存在", file_id)));
        }
        let meta = file_metas.into_one().unwrap();
        return Ok(meta.into());
    }

    async fn list(
        &mut self,
        parent_file_id: &str,
        cloud_meta: CloudMeta,
    ) -> ResponseResult<FileItemWrapper> {
        let drive_id = self.get_drive_id(cloud_meta).await?;
        let query = Query {
            drive_id,
            parent_file_id: parent_file_id.to_string(),
            limit: 100,
            all: false,
            url_expire_sec: 1600,
            image_thumbnail_process: "image/resize,w_400/format,jpeg".to_string(),
            image_url_process: "image/resize,w_1920/format,jpeg".to_string(),
            video_thumbnail_process: "video/snapshot,t_1000,f_jpg,ar_auto,w_300".to_string(),
            fields: "*".to_string(),
            order_by: "updated_at".to_string(),
            order_direction: "DESC".to_string(),
        };

        let resp_result = self
            .inner
            .api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, "adrive/v3/file/list"))
            .json(&query)
            .send();

        let json = self.inner.get_response_text(resp_result).await?;
        // println!("{:?}", json);
        let result = serde_json::from_str(json.as_str());
        return Ok(result.unwrap());
    }
    async fn refresh_token(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<String> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());

        let token_option = cloud_meta.token.clone();
        let token = token_option.unwrap();
        let token: Token = serde_json::from_str(token.as_str()).unwrap();
        // let mut refresh_token = HashMap::new();
        // refresh_token.insert("refresh_token", token.refresh_token);
        let url = format!("oauth/2.0/token?grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}", token.refresh_token, "", "");

        let resp_result = self
            .inner
            .api_client
            .get(format!("{}/{}", AUTH_DOMAIN_PREFIX, url))
            .send();
        let resp_result = self.inner.get_response_text(resp_result).await?;
        Ok(resp_result)
    }

    async fn drive_quota(&mut self, cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let result = self
            .inner
            .get_json(
                format!("api/quota?checkfree=1&checkexpire=1").as_str(),
                &mut extensions,
            )
            .await
            .unwrap();

        let result: BaiduQuota = serde_json::from_str(result.as_str()).unwrap();
        Ok(result.into())
    }

    fn authorize(&self, server: &str, id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        Ok(format!("{}/oauth/2.0/authorize?response_type=code&client_id={}&redirect_uri={}&scope=basic,netdisk&state={}", AUTH_DOMAIN_PREFIX, "iWjfcOWq0BoUNZABxy4hGtXPdftzPtG8", encoded, id))
    }

    async fn callback(&self, server: String, code: String, _id: i32) -> ResponseResult<String> {
        let callback = format!("http://{}/api/cloud/callback", server);
        let encoded = encode(callback.as_str());
        let token_url = format!("{}/{}", AUTH_DOMAIN_PREFIX, format!("oauth/2.0/token?grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}", code, "iWjfcOWq0BoUNZABxy4hGtXPdftzPtG8", "KqEOL6F9tT2vkeeYRgKqZvyPHlGQnujM", encoded));
        info!("{}", token_url);
        let resp_result = self.inner.content_client.get(token_url).send();
        let json_text = self.inner.get_response_text(resp_result).await?;
        info!("{}", json_text);
        Ok(String::from(json_text))
    }

    async fn after_callback(&mut self, cloud_meta: &mut CloudMeta) -> ResponseResult<()> {
        cloud_meta.data_root = Some("/app/share-desk".to_string());
        cloud_meta.status = MetaStatus::Enable.into();
        Ok(())
    }
}

impl Network for Inner {}

impl Inner {
    async fn get_json(
        &mut self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let resp_result = self
            .api_client
            .get(format!("{}/{}", API_DOMAIN_PREFIX, path))
            .build()
            .unwrap();
        let resp_result = self
            .api_client
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
    }
    ///
    ///
    ///
    async fn post_form(
        &mut self,
        path: &str,
        form: &Vec<(&str, &str)>,
        extensions: &mut Extensions,
    ) -> ResponseResult<String> {
        let resp_result = self
            .api_client
            .post(format!("{}/{}", API_DOMAIN_PREFIX, path))
            .form(form)
            .build()
            .unwrap();
        let resp_result = self
            .api_client
            .execute_with_extensions(resp_result, extensions);
        return self.get_response_text(resp_result).await;
    }

    async fn get_bytes(
        &mut self,
        path: &str,
        extensions: &mut Extensions,
    ) -> ResponseResult<Bytes> {
        let resp_result = self
            .api_client
            .get(path)
            .header("User-Agent", "pan.baidu.com")
            .build()
            .unwrap();
        let resp_result = self
            .api_client
            .execute_with_extensions(resp_result, extensions);
        return self.get_request_bytes(resp_result).await;
    }

    ///
    /// 获得用户信息
    ///
    async fn user_info(&mut self, cloud_meta: CloudMeta) -> ResponseResult<User> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let json = self
            .get_json("rest/2.0/xpan/nas?method=uinfo", &mut extensions)
            .await?;
        debug!("{}", json);
        let result: BaiduUser = serde_json::from_str(json.as_str()).unwrap();
        let result: User = result.into();
        self.user = Some(result.clone());
        return Ok(result);
    }
    ///
    /// 管理文件
    ///
    async fn manage_file<T>(
        &mut self,
        opera: BaiduOpera,
        file_list: &Vec<T>,
        async_type: AsyncType,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<BaiduFileManagerResult>
        where
            T: Serialize,
    {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        let mut parameter = vec![];
        let file_list = serde_json::to_string(file_list);
        let file_list = file_list.unwrap();
        let file_list = file_list.as_str();

        parameter.push(("filelist", file_list));
        parameter.push(("async", async_type.into()));
        let opera: &str = opera.into();
        let url = format!("rest/2.0/xpan/file?method=filemanager&opera={}", opera);
        let json = self
            .post_form(url.as_str(), &parameter, &mut extensions)
            .await?;
        debug!("{}", json);
        let result: BaiduFileManagerResult = serde_json::from_str(json.as_str()).unwrap();
        return Ok(result);
    }
    async fn download(&mut self, dlink: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        let mut extensions = Extensions::new();
        extensions.insert(cloud_meta.clone());
        self.get_bytes(dlink, &mut extensions).await
    }
}