use serde::{Deserialize, Serialize};

///
///  {
/// "token_type":"bearer",
/// "expires_in":3600,
/// "scope":"Googledrive.readwrite",
/// "access_token":"EwA4A61DBAAUZUYqEwmYbaXIJECvBbNriwl1aGYAAeTJu9NHOgN0Hfm4Czeo86IsfPp1X+hQupXWMF2D4wNjByRhW/onYX6/N2uW75RJ7b65NSO6q09lBk8YZgL2KJ2zy5qq3OEQmH0X736AuR8WzsouXquFRPQejT+p2wXkjSiel3VPNmy9d/rgfUg2/2dB7unM6e1aUB48Uh/mtGOhn4rc8waFcD44Ohcst9GpcF6uSNSEjDh7cl08LhMtV5Bxfe/wWGxth3oijxrzIx8F21bIwejORpcKcuROHUVAIehASuFJRNOrbmbaBOCCU7n8PibVRx0ppY40Eec9Jcv0rqyGVIL8yKl3sRByqHEsPGlt6icCEwKn4kICMCb6T1UDZgAACLVDVGack+4pCALTTS8Z1Mmhe272BXasIVjBGLdO9qgF1jSXNFmSk4GKxmRR4D5MmLZqUrlHZ9oPi0dfwdaRIvHCJZ5xSWbCnDXsOLBdicuilwQ9dxTyQeGbV7AFWli6aA/mHpmyYkICp0HSJ0rF3N/QiO9FypTZzjO5tMzbIkEbdDdodeN9hX3t/7+g+nL9e7LVUwVEFHHOkju7dF4Xqjm59NPkwPSmIzng4gmm97DRJbcQ25NvLetmvyhhAUhb5A7d0+bDUZ5n4EVK6Ymzsx7JK7NCQiGi5z1HILz2Eiv2GXHHE9W4qG7o7tWiJCfuSlwXLaSWHO4WYjp/8/PWam2xvh50oNcAi/+Sna2ll/HJEvboJZdt4xX+GoXuk3nU1+RMwrvtjKOI3Mv9/snOcb9iDlsPkLWJAS6RSWEU1nAo0hXasrV6wj+EnV6yPS1yNWxxE8gdQeh+TcFy2Fdtc5jHibUVIWM8AycmMnMQdPqEnmXbtyTRAl2fzUd3Z/JK/TVrIA6o7d9NB0BH4y9NgvuNW5OI30d/FOGXfPAm8DF8XKcYd5XBz9XmVD+HSdeljOqqwgHCXIyhYG3Jh8fX9UaC/Lf6dvPRYD9qSBK9aRMrgZyYkmoMcqMDIvVoNcmfMWjd3oqOjrSgu3ycPXmLkRqulhF1KvnYqNoFh1LmubGjFugZ8LuyNPPHYr7Ervt961RGMQI=","authentication_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiIsImtpZCI6IjEifQ.eyJ2ZXIiOjEsImlzcyI6InVybjp3aW5kb3dzOmxpdmVpZCIsImV4cCI6MTY4NTM0NTMwMCwidWlkIjoiQUFBQUFBQUFBQUFBQUFBQUFBQUFBR0xRNjc1aF9kU1pNOVR2Q09PbmhabyIsImF1ZCI6ImFwaTovL2RlNmU0ZGRlLTViNDQtNDM2OC1hNDBkLTI1MjIyYTRlMWU1MSIsInVybjptaWNyb3NvZnQ6YXBwdXJpIjoiYXBwaWQ6Ly9kZTZlNGRkZS01YjQ0LTQzNjgtYTQwZC0yNTIyMmE0ZTFlNTEiLCJ1cm46bWljcm9zb2Z0OmFwcGlkIjoiMDAwMDAwMDA0NEYzQzVGQyJ9.i8R5QZ8JLWyK01EhL_5mOx1Ejyhd9MN-_9qV_oJBNq0","user_id":"AAAAAAAAAAAAAAAAAAAAAGLQ675h_dSZM9TvCOOnhZo"}
///
///
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AuthorizationToken {
    pub(crate) token_type: Option<String>,
    pub(crate) expires_in: Option<u64>,
    scope: Option<String>,
    pub(crate) access_token: Option<String>,
    authentication_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct About {
    pub(crate) kind: Option<String>,
    #[serde(rename = "storageQuota")]
    pub(crate) storage_quota: GoogleDriveQuota,
}
// #[derive(Serialize, Deserialize, Debug)]
// pub(crate) struct Drive {
//     // pub(crate) owner: GoogleDriveOwner,
//     pub(crate) quota: GoogleDriveQuota,
// }

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GoogleDriveQuota {
    pub(crate) usage: String,
    pub(crate) limit: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct GoogleDriveFile {
    pub(crate) kind: Option<String>,
    #[serde(rename = "driveId")]
    pub(crate) drive_id: Option<String>,
    #[serde(rename = "mimeType")]
    pub(crate) mime_type: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    #[serde(rename = "webContentLink")]
    pub(crate) web_content_link: Option<String>,
    pub(crate) parents: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct GoogleDriveExtra {
    // pub(crate) drive_id: Option<String>,
    pub(crate) root_file_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Files {
    pub(crate) files: Vec<GoogleDriveFile>,
}