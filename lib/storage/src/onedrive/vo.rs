use serde::{Deserialize, Serialize};

///
///  {
/// "token_type":"bearer",
/// "expires_in":3600,
/// "scope":"onedrive.readwrite",
/// "access_token":"EwA4A61DBAAUZUYqEwmYbaXIJECvBbNriwl1aGYAAeTJu9NHOgN0Hfm4Czeo86IsfPp1X+hQupXWMF2D4wNjByRhW/onYX6/N2uW75RJ7b65NSO6q09lBk8YZgL2KJ2zy5qq3OEQmH0X736AuR8WzsouXquFRPQejT+p2wXkjSiel3VPNmy9d/rgfUg2/2dB7unM6e1aUB48Uh/mtGOhn4rc8waFcD44Ohcst9GpcF6uSNSEjDh7cl08LhMtV5Bxfe/wWGxth3oijxrzIx8F21bIwejORpcKcuROHUVAIehASuFJRNOrbmbaBOCCU7n8PibVRx0ppY40Eec9Jcv0rqyGVIL8yKl3sRByqHEsPGlt6icCEwKn4kICMCb6T1UDZgAACLVDVGack+4pCALTTS8Z1Mmhe272BXasIVjBGLdO9qgF1jSXNFmSk4GKxmRR4D5MmLZqUrlHZ9oPi0dfwdaRIvHCJZ5xSWbCnDXsOLBdicuilwQ9dxTyQeGbV7AFWli6aA/mHpmyYkICp0HSJ0rF3N/QiO9FypTZzjO5tMzbIkEbdDdodeN9hX3t/7+g+nL9e7LVUwVEFHHOkju7dF4Xqjm59NPkwPSmIzng4gmm97DRJbcQ25NvLetmvyhhAUhb5A7d0+bDUZ5n4EVK6Ymzsx7JK7NCQiGi5z1HILz2Eiv2GXHHE9W4qG7o7tWiJCfuSlwXLaSWHO4WYjp/8/PWam2xvh50oNcAi/+Sna2ll/HJEvboJZdt4xX+GoXuk3nU1+RMwrvtjKOI3Mv9/snOcb9iDlsPkLWJAS6RSWEU1nAo0hXasrV6wj+EnV6yPS1yNWxxE8gdQeh+TcFy2Fdtc5jHibUVIWM8AycmMnMQdPqEnmXbtyTRAl2fzUd3Z/JK/TVrIA6o7d9NB0BH4y9NgvuNW5OI30d/FOGXfPAm8DF8XKcYd5XBz9XmVD+HSdeljOqqwgHCXIyhYG3Jh8fX9UaC/Lf6dvPRYD9qSBK9aRMrgZyYkmoMcqMDIvVoNcmfMWjd3oqOjrSgu3ycPXmLkRqulhF1KvnYqNoFh1LmubGjFugZ8LuyNPPHYr7Ervt961RGMQI=","authentication_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiIsImtpZCI6IjEifQ.eyJ2ZXIiOjEsImlzcyI6InVybjp3aW5kb3dzOmxpdmVpZCIsImV4cCI6MTY4NTM0NTMwMCwidWlkIjoiQUFBQUFBQUFBQUFBQUFBQUFBQUFBR0xRNjc1aF9kU1pNOVR2Q09PbmhabyIsImF1ZCI6ImFwaTovL2RlNmU0ZGRlLTViNDQtNDM2OC1hNDBkLTI1MjIyYTRlMWU1MSIsInVybjptaWNyb3NvZnQ6YXBwdXJpIjoiYXBwaWQ6Ly9kZTZlNGRkZS01YjQ0LTQzNjgtYTQwZC0yNTIyMmE0ZTFlNTEiLCJ1cm46bWljcm9zb2Z0OmFwcGlkIjoiMDAwMDAwMDA0NEYzQzVGQyJ9.i8R5QZ8JLWyK01EhL_5mOx1Ejyhd9MN-_9qV_oJBNq0","user_id":"AAAAAAAAAAAAAAAAAAAAAGLQ675h_dSZM9TvCOOnhZo"}
///
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AuthorizationToken {
    pub(crate) token_type: String,
    pub(crate) expires_in: u64,
    scope: String,
    pub(crate) access_token: String,
    authentication_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    user_id: Option<String>,
}

///
/// {
//     "@odata.context": "https://graph.microsoft.com/v1.0/$metadata#drives/$entity",
//     "driveType": "personal",
//     "id": "5fa1b8314557f6b7",
//     "owner": {
//         "user": {
//             "displayName": "丁 寒",
//             "id": "5fa1b8314557f6b7"
//         }
//     },
//     "quota": {
//         "deleted": 522225,
//         "remaining": 3989778820,
//         "state": "normal",
//         "total": 5368709120,
//         "used": 1378930300,
//         "storagePlanInformation": {
//             "upgradeAvailable": true
//         }
//     }
// }
///
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Drive {
    pub(crate) owner: OneDriveOwner,
    pub(crate) quota: OneDriveQuota,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveQuota {
    deleted: u64,
    pub(crate) used: u64,
    file_count: Option<u64>,
    pub(crate) remaining: u64,
    state: String,
    pub(crate) total: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveOwner {
    pub(crate) user: OneDriveUser,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct OneDriveUser {
    pub(crate) id: String,
    #[serde(rename = "displayName")]
    pub(crate) display_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DriveItem {
    pub(crate) id: String,
    pub(crate) name: String,
}
