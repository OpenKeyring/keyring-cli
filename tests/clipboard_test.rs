#[cfg(target_os = "linux")]
use keyring_cli::clipboard::linux::LinuxClipboard;

#[cfg(target_os = "macos")]
use keyring_cli::clipboard::macos::MacOSClipboard;

#[cfg(target_os = "windows")]
use keyring_cli::clipboard::windows::WindowsClipboard;

use keyring_cli::clipboard::manager::{ClipboardConfig, ClipboardManager};
use keyring_cli::clipboard::ClipboardService;
use std::time::Duration;

// Import serial_test for test serialization
use serial_test::serial;

#[cfg(target_os = "macos")]
#[serial]
#[test]
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
#[serial]
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
#[serial]
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
#[serial]
#[test]
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
#[serial]
#[test]
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

#[cfg(target_os = "macos")]
#[serial]
#[test]
fn test_auto_clear_after_timeout() {
    let macos_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let config = ClipboardConfig {
        timeout_seconds: 2,  // Short timeout for testing
        clear_after_copy: true,
        max_content_length: 256,
    };

    let mut service = ClipboardService::new(macos_clipboard, config);

    // Copy password to clipboard
    assert!(service.copy_password("auto_clear_test").is_ok());

    // Immediately verify content exists
    let content = service.get_clipboard_content();
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "auto_clear_test");

    // Wait for timeout to pass (2 seconds + small buffer)
    std::thread::sleep(Duration::from_secs(3));

    // Verify clipboard was automatically cleared
    // Create a new clipboard instance to check current state
    let mut check_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let result = check_clipboard.get_content();

    // Clipboard should be empty or contain different content
    match result {
        Ok(content) => {
            // If there's content, it should NOT be our test password
            assert_ne!(content, "auto_clear_test",
                       "Clipboard should have been auto-cleared after timeout");
        }
        Err(_) => {
            // Empty clipboard is also acceptable (depends on platform behavior)
        }
    }
}

#[cfg(target_os = "macos")]
#[serial]
#[test]
fn test_no_auto_clear_when_disabled() {
    let macos_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let config = ClipboardConfig {
        timeout_seconds: 1,  // Short timeout
        clear_after_copy: false,  // Auto-clear disabled
        max_content_length: 256,
    };

    let mut service = ClipboardService::new(macos_clipboard, config);

    // Copy password to clipboard
    assert!(service.copy_password("no_clear_test").is_ok());

    // Wait longer than timeout
    std::thread::sleep(Duration::from_secs(2));

    // Content should STILL be there (auto-clear was disabled)
    let mut check_clipboard = MacOSClipboard::new().expect("Failed to create MacOSClipboard");
    let result = check_clipboard.get_content();

    assert!(result.is_ok(), "Clipboard should still have content when clear_after_copy is false");
    assert_eq!(result.unwrap(), "no_clear_test",
               "Content should persist when clear_after_copy is disabled");

    // Clean up manually
    let _ = check_clipboard.clear();
}

#[test]
fn test_clipboard_config_default() {
    let config = ClipboardConfig::default();
    assert_eq!(config.timeout_seconds, 30);
    assert!(config.clear_after_copy);
    assert_eq!(config.max_content_length, 1024);
}
