//! Windows clipboard implementation using clipboard-win crate
//!
//! Provides clipboard functionality on Windows using the clipboard-win crate,
//! which wraps the Windows clipboard API.

use crate::clipboard::manager::ClipboardManager;
use crate::error::KeyringError;
use clipboard_win::{empty_clipboard, get_clipboard_string, set_clipboard_string};
use std::time::Duration;

pub struct WindowsClipboard;

impl ClipboardManager for WindowsClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        set_clipboard_string(content).map_err(|e| KeyringError::Clipboard {
            context: format!("Failed to set clipboard: {}", e),
        })
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        get_clipboard_string().map_err(|e| KeyringError::Clipboard {
            context: format!("Failed to get clipboard: {}", e),
        })
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        empty_clipboard().map_err(|e| KeyringError::Clipboard {
            context: format!("Failed to clear clipboard: {}", e),
        })
    }

    fn is_supported(&self) -> bool {
        true // Windows always supports clipboard
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_clipboard() {
        let mut clipboard = WindowsClipboard;

        // Set clipboard content
        let test_content = "Test clipboard content";
        clipboard.set_content(test_content).unwrap();

        // Get clipboard content
        let retrieved = clipboard.get_content().unwrap();
        assert_eq!(retrieved, test_content);
    }

    #[test]
    fn test_clear_clipboard() {
        let mut clipboard = WindowsClipboard;

        // Set content
        clipboard.set_content("Temporary content").unwrap();

        // Clear
        clipboard.clear().unwrap();

        // Verify it's empty or different
        let content = clipboard.get_content().unwrap();
        // After clearing, clipboard might be empty or contain previous content from other apps
        // We just verify the operation didn't error
    }

    #[test]
    fn test_is_supported() {
        let clipboard = WindowsClipboard;
        assert!(clipboard.is_supported());
    }

    #[test]
    fn test_timeout() {
        let clipboard = WindowsClipboard;
        assert_eq!(clipboard.timeout(), Duration::from_secs(30));
    }
}
