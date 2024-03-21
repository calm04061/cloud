use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use async_ssh2_lite::AsyncSession;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{AsyncReadExt, AsyncWriteExt};

use api::ResponseResult;
use persistence::{CloudMeta, FileBlockMeta};

use crate::sftp::vo::HostUser;
use crate::storage::{AuthMethod, CreateResponse, Quota, Storage};

///
/// 存储文件到sftp
/// cloud_file_id 为 文件路径
/// 文件路径生成规则
///
///
pub(crate) struct SftpStorage {}

impl SftpStorage {
    pub fn new() -> Self {
        SftpStorage {}
    }
}


#[async_trait]
impl Storage for SftpStorage {
    async fn upload_content(
        &mut self,
        file_block: &FileBlockMeta,
        content: &Vec<u8>,
        cloud_meta: &CloudMeta,
    ) -> ResponseResult<CreateResponse> {
        let user: HostUser = cloud_meta.auth.clone().unwrap().into();
        let port = user.port.parse::<u16>().unwrap();
        let addr1 = IpAddr::from_str(user.hostname.as_str()).unwrap();
        let addr = SocketAddr::new(addr1, port);
        let mut session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(addr, None).await?;
        session.handshake().await?;
        session.userauth_password(user.username.as_str(), user.password.expect("not set password").as_str()).await?;
        let remote_path = PathBuf::from(cloud_meta.data_root.clone().unwrap()).join(file_block.file_part_id.clone());
        let mut remote_file = session.scp_send(remote_path.as_path(),
                                               0o644, content.len() as u64, None).await?;

        remote_file.write_all(content.as_slice()).await?;
        remote_file.send_eof().await?;
        remote_file.wait_eof().await?;
        remote_file.close().await?;
        remote_file.wait_close().await?;

        let cloud_file_id = remote_path.display().to_string();
        Ok(CreateResponse {
            encrypt_mode: "".to_string(),
            file_id: cloud_file_id,
            file_name: file_block.file_part_id.clone(),
            file_type: "".to_string(),
        })
    }

    async fn delete(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<()> {
        let user: HostUser = cloud_meta.auth.clone().unwrap().into();
        let port = user.port.parse::<u16>().unwrap();
        let addr1 = IpAddr::from_str(user.hostname.as_str()).unwrap();
        let addr = SocketAddr::new(addr1, port);
        let session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(addr, None).await?;
        session.userauth_password(user.username.as_str(), user.password.expect("not set password").as_str()).await?;
        let sftp = session.sftp().await?;
        let remote_path = PathBuf::from(cloud_file_id);
        sftp.unlink(remote_path.as_path()).await?;
        Ok(())
    }

    async fn content(&mut self, cloud_file_id: &str, cloud_meta: &CloudMeta) -> ResponseResult<Bytes> {
        let user: HostUser = cloud_meta.auth.clone().unwrap().into();
        let port = user.port.parse::<u16>().unwrap();
        let addr1 = IpAddr::from_str(user.hostname.as_str()).unwrap();
        let addr = SocketAddr::new(addr1, port);
        let session = AsyncSession::<async_ssh2_lite::AsyncIoTcpStream>::connect(addr, None).await?;
        session.userauth_password(user.username.as_str(), user.password.expect("not set password").as_str()).await?;
        let remote_path = PathBuf::from(cloud_file_id);
        let (mut remote_file, _stat) = session.scp_recv(remote_path.as_path()).await?;
        let mut buf = vec![];
        remote_file.read_to_end(&mut buf).await?;

        remote_file.send_eof().await?;
        remote_file.wait_eof().await?;
        remote_file.close().await?;
        remote_file.wait_close().await?;
        let bytes_mut = Bytes::from(buf);
        Ok(bytes_mut)
    }

    async fn drive_quota(&mut self, _cloud_meta: &CloudMeta) -> ResponseResult<Quota> {
        Ok(Quota {
            total: 10000000000,
            used: 10000,
            remaining: 10000000000,
        })
    }
    fn get_auth_methods(&self) -> Vec<AuthMethod> {
        vec![AuthMethod::UsernamePassword]
    }

    async fn refresh_token(&mut self, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        todo!()
    }

    fn authorize(&self, _server: &str, _id: i32) -> ResponseResult<String> {
        todo!()
    }

    async fn callback(&self, _server: String, _code: String, _cloud_meta: &mut CloudMeta) -> ResponseResult<String> {
        todo!()
    }
}

