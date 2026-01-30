//! Cloud Provider Configuration
//!
//! Defines the supported cloud providers and their configuration options.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported cloud storage providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    /// iCloud Drive (macOS/iOS)
    #[default]
    ICloud,
    /// Dropbox
    Dropbox,
    /// Google Drive
    GDrive,
    /// Microsoft OneDrive
    OneDrive,
    /// Generic WebDAV
    WebDAV,
    /// SFTP
    SFTP,
    /// Aliyun Drive (阿里云盘)
    AliyunDrive,
    /// Aliyun OSS (阿里云对象存储)
    AliyunOSS,
    /// Tencent COS (腾讯云对象存储)
    TencentCOS,
    /// Huawei OBS (华为云对象存储)
    HuaweiOBS,
    /// UpYun (又拍云)
    UpYun,
}

/// Cloud storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Provider type
    #[serde(default)]
    pub provider: CloudProvider,

    /// iCloud Drive path (macOS: ~/Library/Mobile Documents/com~apple~CloudDocs/)
    pub icloud_path: Option<PathBuf>,

    /// WebDAV endpoint URL
    pub webdav_endpoint: Option<String>,
    /// WebDAV username
    pub webdav_username: Option<String>,
    /// WebDAV password
    pub webdav_password: Option<String>,

    /// SFTP host
    pub sftp_host: Option<String>,
    /// SFTP port (default: 22)
    pub sftp_port: Option<u16>,
    /// SFTP username
    pub sftp_username: Option<String>,
    /// SFTP password
    pub sftp_password: Option<String>,
    /// SFTP root path
    pub sftp_root: Option<String>,

    /// Dropbox access token (future implementation)
    pub dropbox_token: Option<String>,

    /// Google Drive access token (future implementation)
    pub gdrive_token: Option<String>,

    /// OneDrive access token (future implementation)
    pub onedrive_token: Option<String>,

    /// Aliyun Drive access token (future implementation)
    pub aliyun_drive_token: Option<String>,

    /// Aliyun OSS endpoint (future implementation)
    pub aliyun_oss_endpoint: Option<String>,
    /// Aliyun OSS bucket name
    pub aliyun_oss_bucket: Option<String>,
    /// Aliyun OSS access key
    pub aliyun_oss_access_key: Option<String>,
    /// Aliyun OSS secret key
    pub aliyun_oss_secret_key: Option<String>,

    /// Tencent COS secret ID
    pub tencent_cos_secret_id: Option<String>,
    /// Tencent COS secret key
    pub tencent_cos_secret_key: Option<String>,
    /// Tencent COS region (e.g., ap-guangzhou)
    pub tencent_cos_region: Option<String>,
    /// Tencent COS bucket name
    pub tencent_cos_bucket: Option<String>,

    /// Huawei OBS access key
    pub huawei_obs_access_key: Option<String>,
    /// Huawei OBS secret key
    pub huawei_obs_secret_key: Option<String>,
    /// Huawei OBS endpoint
    pub huawei_obs_endpoint: Option<String>,
    /// Huawei OBS bucket name
    pub huawei_obs_bucket: Option<String>,

    /// UpYun bucket name
    pub upyun_bucket: Option<String>,
    /// UpYun operator name
    pub upyun_operator: Option<String>,
    /// UpYun password
    pub upyun_password: Option<String>,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            provider: CloudProvider::default(),
            icloud_path: None,
            webdav_endpoint: None,
            webdav_username: None,
            webdav_password: None,
            sftp_host: None,
            sftp_port: Some(22),
            sftp_username: None,
            sftp_password: None,
            sftp_root: None,
            dropbox_token: None,
            gdrive_token: None,
            onedrive_token: None,
            aliyun_drive_token: None,
            aliyun_oss_endpoint: None,
            aliyun_oss_bucket: None,
            aliyun_oss_access_key: None,
            aliyun_oss_secret_key: None,
            tencent_cos_secret_id: None,
            tencent_cos_secret_key: None,
            tencent_cos_region: None,
            tencent_cos_bucket: None,
            huawei_obs_access_key: None,
            huawei_obs_secret_key: None,
            huawei_obs_endpoint: None,
            huawei_obs_bucket: None,
            upyun_bucket: None,
            upyun_operator: None,
            upyun_password: None,
        }
    }
}
