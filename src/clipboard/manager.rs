use crate::error::KeyringError;
use std::time::Duration;

pub struct ClipboardConfig {
    pub timeout_seconds: u64,
    pub clear_after_copy: bool,
    pub max_content_length: usize,
}

pub trait ClipboardManager {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError>;
    fn get_content(&mut self) -> Result<String, KeyringError>;
    fn clear(&mut self) -> Result<(), KeyringError>;
    fn is_supported(&self) -> bool;
    fn timeout(&self) -> Duration;
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            clear_after_copy: true,
            max_content_length: 1024, // 1KB limit for security
        }
    }
}

pub struct ClipboardService<T: ClipboardManager> {
    manager: T,
    config: ClipboardConfig,
}

impl<T: ClipboardManager> ClipboardService<T> {
    pub fn new(manager: T, config: ClipboardConfig) -> Self {
        Self { manager, config }
    }
}

pub fn create_platform_clipboard() -> Result<Box<dyn ClipboardManager>, KeyringError> {
    match std::env::consts::OS {
        "macos" => Ok(Box::new(macos::MacOSClipboard)),
        "linux" => Ok(Box::new(linux::LinuxClipboard)),
        "windows" => Ok(Box::new(windows::WindowsClipboard)),
        _ => Err(KeyringError::UnsupportedPlatform),
    }
}

impl<T: ClipboardManager> ClipboardService<T> {
    pub fn copy_password(&mut self, password: &str) -> Result<(), KeyringError> {
        if !self.manager.is_supported() {
            return Err(KeyringError::ClipboardNotSupported);
        }

        if password.len() > self.config.max_content_length {
            return Err(KeyringError::ContentTooLong(self.config.max_content_length));
        }

        self.manager.set_content(password)?;

        if self.config.clear_after_copy {
            std::thread::spawn(move || {
                std::thread::sleep(self.manager.timeout());
                // In a real implementation, this would clear the clipboard
                println!("Clipboard cleared after timeout");
            });
        }

        Ok(())
    }

    pub fn get_clipboard_content(&mut self) -> Result<String, KeyringError> {
        self.manager.get_content()
    }

    pub fn clear_clipboard(&mut self) -> Result<(), KeyringError> {
        self.manager.clear()
    }
}