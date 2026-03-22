use crate::clipboard::manager::ClipboardManager;
use crate::error::KeyringError;
use std::time::Duration;

pub struct MacOSClipboard {
    clipboard: arboard::Clipboard,
}

impl MacOSClipboard {
    pub fn new() -> Result<Self, KeyringError> {
        arboard::Clipboard::new()
            .map(|clipboard| Self { clipboard })
            .map_err(|e| KeyringError::IoError(format!("Failed to initialize clipboard: {}", e)))
    }
}

impl ClipboardManager for MacOSClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        // SECURITY: arboard uses NSPasteboard API directly (no subprocess)
        // This is secure because:
        // 1. No process arguments exposure
        // 2. No shell history logging risk
        // 3. Direct API call to system clipboard
        self.clipboard
            .set_text(content)
            .map_err(|e| KeyringError::IoError(format!("Failed to set clipboard: {}", e)))
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        // SECURITY: arboard uses NSPasteboard API directly
        self.clipboard
            .get_text()
            .map_err(|e| KeyringError::IoError(format!("Failed to get clipboard: {}", e)))
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        // SECURITY: arboard clears clipboard via API
        self.clipboard
            .clear()
            .map_err(|e| KeyringError::IoError(format!("Failed to clear clipboard: {}", e)))
    }

    fn is_supported(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}
