//! OpenDAL Cloud Storage Provider Tests
//!
//! Integration tests for the cloud storage operator factory.

use keyring_cli::cloud::{config::CloudConfig, provider::create_operator};
use tempfile::TempDir;

#[test]
fn test_icloud_operator_creation() {
    // Create a temporary directory to simulate iCloud Drive
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let icloud_path = temp_dir
        .path()
        .join("Library/Mobile Documents/com~apple~CloudDocs/OpenKeyring");

    // Create the config
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::ICloud,
        icloud_path: Some(icloud_path.clone()),
        ..Default::default()
    };

    // Create the operator
    let result = create_operator(&config);

    // Verify the operator was created successfully
    assert!(
        result.is_ok(),
        "Failed to create iCloud operator: {:?}",
        result.err()
    );

    let operator = result.unwrap();
    assert!(operator.info().full_capability().read);
    assert!(operator.info().full_capability().write);
    assert!(operator.info().full_capability().list);
}

#[test]
fn test_webdav_operator_creation() {
    // Create WebDAV config
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::WebDAV,
        webdav_endpoint: Some("https://dav.example.com/openkeyring".to_string()),
        webdav_username: Some("testuser".to_string()),
        webdav_password: Some("testpass".to_string()),
        ..Default::default()
    };

    // Create the operator (should succeed even if connection fails)
    let result = create_operator(&config);

    // Verify the operator was created successfully
    assert!(
        result.is_ok(),
        "Failed to create WebDAV operator: {:?}",
        result.err()
    );

    let operator = result.unwrap();
    assert!(operator.info().full_capability().read);
    assert!(operator.info().full_capability().write);
}

#[cfg(unix)]
#[test]
fn test_sftp_operator_creation() {
    // Create SFTP config
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::SFTP,
        sftp_host: Some("sftp.example.com".to_string()),
        sftp_username: Some("testuser".to_string()),
        sftp_password: Some("testpass".to_string()),
        sftp_root: Some("/openkeyring".to_string()),
        ..Default::default()
    };

    // Create the operator (should succeed even if connection fails)
    let result = create_operator(&config);

    // Verify the operator was created successfully
    assert!(
        result.is_ok(),
        "Failed to create SFTP operator: {:?}",
        result.err()
    );

    let operator = result.unwrap();
    assert!(operator.info().full_capability().read);
    assert!(operator.info().full_capability().write);
}

#[test]
fn test_unimplemented_provider_returns_error() {
    // Test Dropbox without required token
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::Dropbox,
        dropbox_token: None, // Missing token
        ..Default::default()
    };

    let result = create_operator(&config);
    assert!(result.is_err());
    // Should return error about missing token, not "not implemented"
    assert!(result.unwrap_err().to_string().contains("dropbox_token"));
}

#[test]
fn test_icloud_without_path_returns_error() {
    // Test iCloud without path
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::ICloud,
        icloud_path: None,
        ..Default::default()
    };

    let result = create_operator(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("icloud_path"));
}

#[test]
fn test_webdav_without_endpoint_returns_error() {
    // Test WebDAV without endpoint
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::WebDAV,
        webdav_endpoint: None,
        webdav_username: Some("testuser".to_string()),
        webdav_password: Some("testpass".to_string()),
        ..Default::default()
    };

    let result = create_operator(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("endpoint"));
}

#[cfg(unix)]
#[test]
fn test_sftp_without_host_returns_error() {
    // Test SFTP without host
    let config = CloudConfig {
        provider: keyring_cli::cloud::config::CloudProvider::SFTP,
        sftp_host: None,
        sftp_username: Some("testuser".to_string()),
        sftp_password: Some("testpass".to_string()),
        ..Default::default()
    };

    let result = create_operator(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("host"));
}

#[test]
fn test_sftp_not_supported_on_windows() {
    #[cfg(not(unix))]
    {
        // Test SFTP on Windows returns platform not supported error
        let config = CloudConfig {
            provider: keyring_cli::cloud::config::CloudProvider::SFTP,
            sftp_host: Some("sftp.example.com".to_string()),
            sftp_username: Some("testuser".to_string()),
            sftp_password: Some("testpass".to_string()),
            ..Default::default()
        };

        let result = create_operator(&config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("not supported") || error_msg.contains("platform"),
            "Expected platform support error, got: {}",
            error_msg
        );
    }

    #[cfg(unix)]
    {
        // On Unix, this test is a no-op since SFTP is supported
        // The actual SFTP tests will run
    }
}
