//! Unit tests for ProviderConfigScreen

use super::{ConfigField, ProviderConfig, ProviderConfigScreen};
use crate::cloud::CloudProvider;

#[test]
fn test_config_field_creation() {
    let field = ConfigField::new("Test Field", false);
    assert_eq!(field.label, "Test Field");
    assert!(field.value.is_empty());
    assert!(!field.is_password);
    assert!(!field.is_focused);
}

#[test]
fn test_config_field_password() {
    let field = ConfigField::new("Password", true);
    assert!(field.is_password);
}

#[test]
fn test_provider_config_creation() {
    let config = ProviderConfig::new(CloudProvider::WebDAV);
    assert_eq!(config.provider, CloudProvider::WebDAV);
    assert!(config.values.is_empty());
}

#[test]
fn test_provider_config_set_get() {
    let mut config = ProviderConfig::new(CloudProvider::WebDAV);
    config.set("url", "https://example.com".to_string());
    assert_eq!(config.get("url"), Some(&"https://example.com".to_string()));
    assert_eq!(config.get("missing"), None);
}

#[test]
fn test_screen_creation_icloud() {
    let screen = ProviderConfigScreen::new(CloudProvider::ICloud);
    assert_eq!(screen.provider, CloudProvider::ICloud);
    assert_eq!(screen.fields.len(), 1);
    assert_eq!(screen.focused_index, 0);
}

#[test]
fn test_screen_creation_webdav() {
    let screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    assert_eq!(screen.provider, CloudProvider::WebDAV);
    assert_eq!(screen.fields.len(), 3);
    assert_eq!(screen.fields[0].label, "WebDAV URL");
    assert_eq!(screen.fields[1].label, "Username");
    assert_eq!(screen.fields[2].label, "Password");
}

#[test]
fn test_screen_creation_sftp() {
    let screen = ProviderConfigScreen::new(CloudProvider::SFTP);
    assert_eq!(screen.provider, CloudProvider::SFTP);
    assert_eq!(screen.fields.len(), 5);
    assert_eq!(screen.fields[0].label, "Host");
    assert_eq!(screen.fields[4].label, "Root Path");
}

#[test]
fn test_screen_creation_aliyun_oss() {
    let screen = ProviderConfigScreen::new(CloudProvider::AliyunOSS);
    assert_eq!(screen.provider, CloudProvider::AliyunOSS);
    assert_eq!(screen.fields.len(), 4);
    assert_eq!(screen.fields[0].label, "Endpoint");
    assert_eq!(screen.fields[3].label, "Access Key Secret");
}

#[test]
fn test_screen_creation_tencent_cos() {
    let screen = ProviderConfigScreen::new(CloudProvider::TencentCOS);
    assert_eq!(screen.provider, CloudProvider::TencentCOS);
    assert_eq!(screen.fields.len(), 4);
    assert_eq!(screen.fields[1].label, "Secret Key");
}

#[test]
fn test_get_fields() {
    let screen = ProviderConfigScreen::new(CloudProvider::Dropbox);
    let fields = screen.get_fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].label, "Access Token");
}

#[test]
fn test_get_focused_field_index() {
    let screen = ProviderConfigScreen::new(CloudProvider::Dropbox);
    assert_eq!(screen.get_focused_field_index(), 0);
}

#[test]
fn test_get_field_value_empty() {
    let screen = ProviderConfigScreen::new(CloudProvider::Dropbox);
    // get_field_value returns Some("") for empty fields (not None)
    assert!(screen.get_field_value(0).unwrap().is_empty());
    // Out of bounds returns None
    assert!(screen.get_field_value(1).is_none());
}

#[test]
fn test_get_config() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    screen.fields[0].value = "https://webdav.example.com".to_string();
    screen.fields[1].value = "user".to_string();
    screen.fields[2].value = "pass".to_string();
    let config = screen.get_config();
    assert_eq!(config.provider, CloudProvider::WebDAV);
    assert_eq!(
        config.get("field_0"),
        Some(&"https://webdav.example.com".to_string())
    );
    assert_eq!(config.get("field_1"), Some(&"user".to_string()));
    assert_eq!(config.get("field_2"), Some(&"pass".to_string()));
}

#[test]
fn test_validate_empty_field() {
    let screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    // All non-password fields are empty
    assert!(screen.validate().is_err());
}

#[test]
fn test_validate_with_values() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::Dropbox);
    screen.fields[0].value = "test_token".to_string();
    assert!(screen.validate().is_ok());
}

#[test]
fn test_to_cloud_config_icloud() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::ICloud);
    screen.fields[0].value = "/path/to/vault".to_string();
    let config = screen.to_cloud_config();
    assert!(config.icloud_path.is_some());
}

#[test]
fn test_to_cloud_config_webdav() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    screen.fields[0].value = "https://webdav.example.com".to_string();
    screen.fields[1].value = "user".to_string();
    screen.fields[2].value = "pass".to_string();
    let config = screen.to_cloud_config();
    assert_eq!(
        config.webdav_endpoint,
        Some("https://webdav.example.com".to_string())
    );
    assert_eq!(config.webdav_username, Some("user".to_string()));
    assert_eq!(config.webdav_password, Some("pass".to_string()));
}

#[test]
fn test_to_cloud_config_sftp() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::SFTP);
    screen.fields[0].value = "sftp.example.com".to_string();
    screen.fields[1].value = "22".to_string();
    screen.fields[2].value = "user".to_string();
    screen.fields[3].value = "pass".to_string();
    screen.fields[4].value = "/data".to_string();
    let config = screen.to_cloud_config();
    assert_eq!(config.sftp_host, Some("sftp.example.com".to_string()));
    assert_eq!(config.sftp_port, Some(22));
    assert_eq!(config.sftp_username, Some("user".to_string()));
    assert_eq!(config.sftp_password, Some("pass".to_string()));
    assert_eq!(config.sftp_root, Some("/data".to_string()));
}

#[test]
fn test_to_cloud_config_empty() {
    let screen = ProviderConfigScreen::new(CloudProvider::Dropbox);
    let config = screen.to_cloud_config();
    assert!(config.dropbox_token.is_none());
}
