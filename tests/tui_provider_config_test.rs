//! Provider Configuration Screen Tests

use keyring_cli::cloud::CloudProvider;
use keyring_cli::tui::screens::provider_config::{ProviderConfig, ProviderConfigScreen};

#[test]
fn test_webdav_config_fields() {
    let screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    let fields = screen.get_fields();

    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].label, "WebDAV URL");
    assert_eq!(fields[1].label, "用户名");
    assert_eq!(fields[2].label, "密码");
}

#[test]
fn test_field_navigation() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::WebDAV);

    // Initially focused on first field
    assert_eq!(screen.get_focused_field_index(), 0);

    // Tab to next field
    screen.handle_tab();
    assert_eq!(screen.get_focused_field_index(), 1);

    // Enter text
    screen.handle_char('h');
    screen.handle_char('t');
    screen.handle_char('t');
    screen.handle_char('p');

    assert_eq!(screen.get_field_value(1), Some("http".to_string()));
}

#[test]
fn test_sftp_config_fields() {
    let screen = ProviderConfigScreen::new(CloudProvider::SFTP);
    let fields = screen.get_fields();

    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0].label, "主机");
    assert_eq!(fields[1].label, "端口");
    assert_eq!(fields[2].label, "用户名");
    assert_eq!(fields[3].label, "密码");
}

#[test]
fn test_shift_tab_navigation() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::SFTP);

    // Move to third field
    screen.handle_tab();
    screen.handle_tab();
    assert_eq!(screen.get_focused_field_index(), 2);

    // Shift+Tab back
    screen.handle_shift_tab();
    assert_eq!(screen.get_focused_field_index(), 1);

    // Can't go below 0
    screen.handle_shift_tab();
    screen.handle_shift_tab();
    assert_eq!(screen.get_focused_field_index(), 0);
}

#[test]
fn test_backspace() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::WebDAV);

    // Enter text in first field
    screen.handle_char('h');
    screen.handle_char('e');
    screen.handle_char('l');
    screen.handle_char('l');
    screen.handle_char('o');

    assert_eq!(screen.get_field_value(0), Some("hello".to_string()));

    // Backspace
    screen.handle_backspace();
    assert_eq!(screen.get_field_value(0), Some("hell".to_string()));

    // Backspace multiple times
    screen.handle_backspace();
    screen.handle_backspace();
    assert_eq!(screen.get_field_value(0), Some("he".to_string()));
}

#[test]
fn test_provider_config() {
    let mut screen = ProviderConfigScreen::new(CloudProvider::WebDAV);

    // Fill in some values
    screen.handle_char('u');
    screen.handle_tab();
    screen.handle_char('a');
    screen.handle_tab();
    screen.handle_char('p');

    let config = screen.get_config();
    assert_eq!(config.provider, CloudProvider::WebDAV);
    assert_eq!(config.get("field_0"), Some(&"u".to_string()));
    assert_eq!(config.get("field_1"), Some(&"a".to_string()));
    assert_eq!(config.get("field_2"), Some(&"p".to_string()));
}

#[test]
fn test_password_field_masking() {
    let screen = ProviderConfigScreen::new(CloudProvider::WebDAV);
    let fields = screen.get_fields();

    // Password field should be marked for masking
    assert_eq!(fields[2].is_password, true);

    // Other fields should not be password fields
    assert_eq!(fields[0].is_password, false);
    assert_eq!(fields[1].is_password, false);
}

#[test]
fn test_empty_field_value() {
    let screen = ProviderConfigScreen::new(CloudProvider::SFTP);

    // Empty field should return empty string, not None
    assert_eq!(screen.get_field_value(0), Some("".to_string()));
    assert_eq!(screen.get_field_value(99), None); // Invalid index returns None
}

