use crate::clipboard::manager::ClipboardManager;
use crate::error::KeyringError;
use std::time::Duration;

pub struct MacOSClipboard;

impl ClipboardManager for MacOSClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        // In a real implementation, this would use the clipboard framework
        // For now, we'll simulate it
        println!("[MOCK] Setting macOS clipboard content: {}", content);
        Ok(())
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        // In a real implementation, this would read from the clipboard
        println!("[MOCK] Reading from macOS clipboard");
        Ok("mock_clipboard_content".to_string())
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        // In a real implementation, this would clear the clipboard
        println!("[MOCK] Clearing macOS clipboard");
        Ok(())
    }

    fn is_supported(&self) -> bool {
        true // macOS always supports clipboard
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}