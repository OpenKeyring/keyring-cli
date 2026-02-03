#[cfg(target_os = "linux")]
use keyring_cli::clipboard::linux::LinuxClipboard;

#[cfg(target_os = "macos")]
use keyring_cli::clipboard::macos::MacOSClipboard;

#[cfg(target_os = "windows")]
use keyring_cli::clipboard::windows::WindowsClipboard;

use keyring_cli::clipboard::manager::{ClipboardConfig, ClipboardManager};
use keyring_cli::clipboard::ClipboardService;
use std::time::Duration;

#[cfg(target_os = "macos")]
#[test]
#[ignore = "Requires GUI context - run manually with: cargo test --test clipboard_test -- --ignored"]
fn test_macos_clipboard() {
    let mut clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    assert!(clipboard.is_supported());

    // Test setting content
    assert!(clipboard.set_content("test_password").is_ok());
    assert!(clipboard.get_content().is_ok());
    assert!(clipboard.clear().is_ok());

    assert_eq!(clipboard.timeout(), Duration::from_secs(30));
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_clipboard() {
    let mut clipboard = WindowsClipboard;
    assert!(clipboard.is_supported());

    assert!(clipboard.set_content("test_password").is_ok());
    assert!(clipboard.get_content().is_ok());
    assert!(clipboard.clear().is_ok());

    assert_eq!(clipboard.timeout(), Duration::from_secs(30));
}

#[cfg(target_os = "linux")]
#[test]
fn test_linux_clipboard() {
    // This test will pass if xclip is available
    let mut clipboard = LinuxClipboard;
    let supported = clipboard.is_supported();

    if supported {
        assert!(clipboard.set_content("test_password").is_ok());
        assert!(clipboard.get_content().is_ok());
        assert!(clipboard.clear().is_ok());
    }

    assert_eq!(clipboard.timeout(), Duration::from_secs(45));
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "Requires GUI context - run manually with: cargo test --test clipboard_test -- --ignored"]
fn test_clipboard_service() {
    let macos_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let config = ClipboardConfig {
        timeout_seconds: 60,
        clear_after_copy: true,
        max_content_length: 256,
    };

    let mut service = ClipboardService::new(macos_clipboard, config);

    // Test copying password
    assert!(service.copy_password("test_password").is_ok());

    // Test getting content
    assert!(service.get_clipboard_content().is_ok());

    // Test clearing
    assert!(service.clear_clipboard().is_ok());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "Requires GUI context - run manually with: cargo test --test clipboard_test -- --ignored"]
fn test_content_length_limit() {
    let macos_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let config = ClipboardConfig {
        timeout_seconds: 30,
        clear_after_copy: true,
        max_content_length: 10,
    };

    let mut service = ClipboardService::new(macos_clipboard, config);

    // Should fail with long content
    let long_password = "a".repeat(20);
    assert!(service.copy_password(&long_password).is_err());
}

#[test]
fn test_clipboard_config_default() {
    let config = ClipboardConfig::default();
    assert_eq!(config.timeout_seconds, 30);
    assert!(config.clear_after_copy);
    assert_eq!(config.max_content_length, 1024);
}
