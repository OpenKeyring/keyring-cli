use crate::clipboard::manager::ClipboardManager;
use crate::error::KeyringError;
use std::time::Duration;

pub struct WindowsClipboard;

impl ClipboardManager for WindowsClipboard {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        // In a real implementation, this would use Windows clipboard API
        // For now, we'll simulate it
        println!("[MOCK] Setting Windows clipboard content: {}", content);
        Ok(())
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        // In a real implementation, this would read from Windows clipboard
        println!("[MOCK] Reading from Windows clipboard");
        Ok("mock_clipboard_content".to_string())
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        // In a real implementation, this would clear the Windows clipboard
        println!("[MOCK] Clearing Windows clipboard");
        Ok(())
    }

    fn is_supported(&self) -> bool {
        true // Windows always supports clipboard
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}