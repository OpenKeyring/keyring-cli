//! Cloud Storage Operator Factory
//!
//! Creates OpenDAL operators for various cloud storage providers.

use crate::cloud::config::{CloudConfig, CloudProvider};
use anyhow::{Context, Result};
use opendal::Operator;

/// Creates an OpenDAL operator based on the provided cloud configuration
///
/// # Arguments
///
/// * `config` - Cloud provider configuration
///
/// # Returns
///
/// Returns a configured `Operator` instance or an error if configuration is invalid
///
/// # Examples
///
/// ```no_run
/// use keyring_cli::cloud::{config::CloudConfig, provider::create_operator};
///
/// let config = CloudConfig {
///     provider: keyring_cli::cloud::config::CloudProvider::ICloud,
///     icloud_path: Some("/path/to/icloud".into()),
///     ..Default::default()
/// };
///
/// let operator = create_operator(&config)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn create_operator(config: &CloudConfig) -> Result<Operator> {
    match config.provider {
        CloudProvider::ICloud => create_icloud_operator(config),
        CloudProvider::WebDAV => create_webdav_operator(config),
        CloudProvider::SFTP => create_sftp_operator(config),
        CloudProvider::Dropbox => create_dropbox_operator(config),
        CloudProvider::GDrive => create_gdrive_operator(config),
        CloudProvider::OneDrive => create_onedrive_operator(config),
        CloudProvider::AliyunDrive => create_aliyun_drive_operator(config),
        CloudProvider::AliyunOSS => create_aliyun_oss_operator(config),
        CloudProvider::TencentCOS => create_tencent_cos_operator(config),
        CloudProvider::HuaweiOBS => create_huawei_obs_operator(config),
        CloudProvider::UpYun => create_upyun_operator(config),
    }
}

/// Creates an operator for iCloud Drive using the Fs service
fn create_icloud_operator(config: &CloudConfig) -> Result<Operator> {
    let path = config
        .icloud_path
        .as_ref()
        .context("icloud_path is required for ICloud provider")?;

    // Use OpenDAL's Fs service to access the local iCloud Drive path
    let builder = opendal::services::Fs::default()
        .root(path.to_string_lossy().as_ref());

    let operator = Operator::new(builder)
        .context("Failed to build Fs operator for iCloud Drive")?
        .finish();

    Ok(operator)
}

/// Creates an operator for WebDAV
fn create_webdav_operator(config: &CloudConfig) -> Result<Operator> {
    let endpoint = config
        .webdav_endpoint
        .as_ref()
        .context("webdav_endpoint is required for WebDAV provider")?;

    let username = config
        .webdav_username
        .as_ref()
        .context("webdav_username is required for WebDAV provider")?;

    let password = config
        .webdav_password
        .as_ref()
        .context("webdav_password is required for WebDAV provider")?;

    let builder = opendal::services::Webdav::default()
        .endpoint(endpoint)
        .username(username)
        .password(password);

    let operator = Operator::new(builder)
        .context("Failed to build WebDAV operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for SFTP
fn create_sftp_operator(config: &CloudConfig) -> Result<Operator> {
    let host = config
        .sftp_host
        .as_ref()
        .context("sftp_host is required for SFTP provider")?;

    let username = config
        .sftp_username
        .as_ref()
        .context("sftp_username is required for SFTP provider")?;

    let password = config
        .sftp_password
        .as_ref()
        .context("sftp_password is required for SFTP provider")?;

    let mut builder = opendal::services::Sftp::default()
        .endpoint(host.as_str())
        .user(username)
        .key(password); // SFTP uses 'key' for password authentication

    // Set root path if provided
    if let Some(root) = &config.sftp_root {
        builder = builder.root(root);
    }

    let operator = Operator::new(builder)
        .context("Failed to build SFTP operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Dropbox
fn create_dropbox_operator(config: &CloudConfig) -> Result<Operator> {
    let token = config
        .dropbox_token
        .as_ref()
        .context("dropbox_token is required for Dropbox provider")?;

    let builder = opendal::services::Dropbox::default()
        .access_token(token)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Dropbox operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Google Drive
fn create_gdrive_operator(config: &CloudConfig) -> Result<Operator> {
    let token = config
        .gdrive_token
        .as_ref()
        .context("gdrive_token is required for Google Drive provider")?;

    let builder = opendal::services::Gdrive::default()
        .access_token(token)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Google Drive operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for OneDrive
fn create_onedrive_operator(config: &CloudConfig) -> Result<Operator> {
    let token = config
        .onedrive_token
        .as_ref()
        .context("onedrive_token is required for OneDrive provider")?;

    let builder = opendal::services::Onedrive::default()
        .access_token(token)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build OneDrive operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Aliyun Drive
fn create_aliyun_drive_operator(config: &CloudConfig) -> Result<Operator> {
    let token = config
        .aliyun_drive_token
        .as_ref()
        .context("aliyun_drive_token is required for Aliyun Drive provider")?;

    let builder = opendal::services::AliyunDrive::default()
        .refresh_token(token)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Aliyun Drive operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Aliyun OSS
fn create_aliyun_oss_operator(config: &CloudConfig) -> Result<Operator> {
    let endpoint = config
        .aliyun_oss_endpoint
        .as_ref()
        .context("aliyun_oss_endpoint is required for Aliyun OSS provider")?;
    let bucket = config
        .aliyun_oss_bucket
        .as_ref()
        .context("aliyun_oss_bucket is required for Aliyun OSS provider")?;
    let access_key = config
        .aliyun_oss_access_key
        .as_ref()
        .context("aliyun_oss_access_key is required for Aliyun OSS provider")?;
    let secret_key = config
        .aliyun_oss_secret_key
        .as_ref()
        .context("aliyun_oss_secret_key is required for Aliyun OSS provider")?;

    let builder = opendal::services::Oss::default()
        .endpoint(endpoint)
        .bucket(bucket)
        .access_key_id(access_key)
        .access_key_secret(secret_key)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Aliyun OSS operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Tencent COS
fn create_tencent_cos_operator(config: &CloudConfig) -> Result<Operator> {
    let secret_id = config
        .tencent_cos_secret_id
        .as_ref()
        .context("tencent_cos_secret_id is required for Tencent COS provider")?;
    let secret_key = config
        .tencent_cos_secret_key
        .as_ref()
        .context("tencent_cos_secret_key is required for Tencent COS provider")?;
    let region = config
        .tencent_cos_region
        .as_ref()
        .context("tencent_cos_region is required for Tencent COS provider")?;
    let bucket = config
        .tencent_cos_bucket
        .as_ref()
        .context("tencent_cos_bucket is required for Tencent COS provider")?;

    let endpoint = format!("https://{}.cos.{}.myqcloud.com", bucket, region);
    let builder = opendal::services::Cos::default()
        .endpoint(&endpoint)
        .secret_id(secret_id)
        .secret_key(secret_key)
        .bucket(bucket)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Tencent COS operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for Huawei OBS
fn create_huawei_obs_operator(config: &CloudConfig) -> Result<Operator> {
    let access_key = config
        .huawei_obs_access_key
        .as_ref()
        .context("huawei_obs_access_key is required for Huawei OBS provider")?;
    let secret_key = config
        .huawei_obs_secret_key
        .as_ref()
        .context("huawei_obs_secret_key is required for Huawei OBS provider")?;
    let endpoint = config
        .huawei_obs_endpoint
        .as_ref()
        .context("huawei_obs_endpoint is required for Huawei OBS provider")?;
    let bucket = config
        .huawei_obs_bucket
        .as_ref()
        .context("huawei_obs_bucket is required for Huawei OBS provider")?;

    let builder = opendal::services::Obs::default()
        .endpoint(endpoint)
        .access_key_id(access_key)
        .secret_access_key(secret_key)
        .bucket(bucket)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build Huawei OBS operator")?
        .finish();

    Ok(operator)
}

/// Creates an operator for UpYun
fn create_upyun_operator(config: &CloudConfig) -> Result<Operator> {
    let bucket = config
        .upyun_bucket
        .as_ref()
        .context("upyun_bucket is required for UpYun provider")?;
    let operator_name = config
        .upyun_operator
        .as_ref()
        .context("upyun_operator is required for UpYun provider")?;
    let password = config
        .upyun_password
        .as_ref()
        .context("upyun_password is required for UpYun provider")?;

    let builder = opendal::services::Upyun::default()
        .bucket(bucket)
        .operator(operator_name)
        .password(password)
        .root("/");

    let operator = Operator::new(builder)
        .context("Failed to build UpYun operator")?
        .finish();

    Ok(operator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider_default() {
        let provider = CloudProvider::default();
        assert_eq!(provider, CloudProvider::ICloud);
    }

    #[test]
    fn test_cloud_config_default() {
        let config = CloudConfig::default();
        assert_eq!(config.provider, CloudProvider::ICloud);
        assert!(config.icloud_path.is_none());
        assert!(config.webdav_endpoint.is_none());
        assert!(config.sftp_host.is_none());
        assert_eq!(config.sftp_port, Some(22));
    }
}
