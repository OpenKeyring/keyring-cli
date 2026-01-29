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
        CloudProvider::Dropbox
        | CloudProvider::GDrive
        | CloudProvider::OneDrive
        | CloudProvider::AliyunDrive
        | CloudProvider::AliyunOSS => {
            anyhow::bail!(
                "Cloud provider {:?} is not implemented yet",
                config.provider
            )
        }
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
