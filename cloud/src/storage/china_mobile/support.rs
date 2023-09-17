use crate::domain::table::tables::CloudMeta;
use crate::error::ErrorInfo;
use crate::storage::china_mobile::vo::{AccessToken, ContentInfo, DelCatalogContent, DiskInfo, GetContentInfo, PcUploadFileRequest, UploadContentInfo};
use crate::storage::storage::{FileInfo, Quota, ResponseResult, TokenProvider};
use crate::util::ToXml;

impl TokenProvider<AccessToken> for CloudMeta {
    fn get_token(&self) -> ResponseResult<AccessToken> {
        let auth = self.auth.clone().unwrap();
        let token = serde_json::from_str(auth.as_str());
        match token {
            Ok(token) => {
                Ok(token)
            }
            Err(e) => {
                return Err(ErrorInfo::OTHER(50, e.to_string()));
            }
        }
    }
}

impl From<DiskInfo> for Quota {
    fn from(value: DiskInfo) -> Self {
        Quota {
            total: value.disk_size,
            used: value.disk_size - value.free_disk_size,
            remaining: value.free_disk_size,
        }
    }
}

impl From<ContentInfo> for FileInfo {
    fn from(_value: ContentInfo) -> Self {
        todo!()
    }
}
impl ToXml for UploadContentInfo {
    fn to_xml(&self, buf: &mut String) {
        buf.push_str("<uploadContentInfo>");
        buf.push_str("<contentName>");
        buf.push_str(self.content_name.as_str());
        buf.push_str("</contentName>");
        buf.push_str("<contentSize>");
        buf.push_str(self.content_size.to_string().as_str());
        buf.push_str("</contentSize>");
        buf.push_str("<contentDesc>");
        buf.push_str(self.content_desc.as_str());
        buf.push_str("</contentDesc>");
        buf.push_str("<contentTagList>");
        buf.push_str(self.content_tag_list.as_str());
        buf.push_str("</contentTagList>");
        buf.push_str("<comlexFlag>");
        buf.push_str(self.comlex_flag.to_string().as_str());
        buf.push_str("</comlexFlag>");
        buf.push_str("<comlexCid>");
        buf.push_str(self.comlex_cid.as_str());
        buf.push_str("</comlexCid>");
        buf.push_str("<resCid>");
        buf.push_str(self.res_cid.as_str());
        buf.push_str("</resCid>");
        buf.push_str("<digest>");
        buf.push_str(self.digest.as_str());
        buf.push_str("</digest>");
        buf.push_str("</uploadContentInfo>");
    }
}

impl ToXml for PcUploadFileRequest {
    fn to_xml(&self, buf: &mut String) {
        buf.push_str("<pcUploadFileRequest>");
        buf.push_str("<totalSize>");
        buf.push_str(self.total_size.to_string().as_str());
        buf.push_str("</totalSize>");
        buf.push_str(format!("<uploadContentList length=\"{}\">", self.upload_content_list.len()).as_str());
        for x in &self.upload_content_list {
            x.to_xml(buf);
        }
        buf.push_str("</uploadContentList>");
        buf.push_str("</pcUploadFileRequest>");
    }
}

impl ToXml for GetContentInfo {
    fn to_xml(&self, _buf: &mut String) {
        todo!()
    }
}
impl ToXml for DelCatalogContent {
    fn to_xml(&self, _buf: &mut String) {
        todo!()
    }
}
