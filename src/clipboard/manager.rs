use crate::error::KeyringError;
use std::time::Duration;

#[cfg(target_os = "macos")]
use crate::clipboard::macos::MacOSClipboard;
#[cfg(target_os = "linux")]
use crate::clipboard::linux::LinuxClipboard;
#[cfg(target_os = "windows")]
use crate::clipboard::windows::WindowsClipboard;

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


// Wrapper for Box<dyn ClipboardManager>
pub struct BoxClipboardManager {
    inner: Box<dyn ClipboardManager>,
}

impl ClipboardManager for BoxClipboardManager {
    fn set_content(&mut self, content: &str) -> Result<(), KeyringError> {
        self.inner.set_content(content)
    }

    fn get_content(&mut self) -> Result<String, KeyringError> {
        self.inner.get_content()
    }

    fn clear(&mut self) -> Result<(), KeyringError> {
        self.inner.clear()
    }

    fn is_supported(&self) -> bool {
        self.inner.is_supported()
    }

    fn timeout(&self) -> Duration {
        self.inner.timeout()
    }
}

impl From<Box<dyn ClipboardManager>> for BoxClipboardManager {
    fn from(inner: Box<dyn ClipboardManager>) -> Self {
        Self { inner }
    }
}

pub fn create_platform_clipboard() -> Result<BoxClipboardManager, KeyringError> {
    #[cfg(target_os = "macos")]
    {
        let clipboard: Box<dyn ClipboardManager> = Box::new(MacOSClipboard);
        Ok(BoxClipboardManager::from(clipboard))
    }
    #[cfg(target_os = "linux")]
    {
        let clipboard: Box<dyn ClipboardManager> = Box::new(LinuxClipboard);
        Ok(BoxClipboardManager::from(clipboard))
    }
    #[cfg(target_os = "windows")]
    {
        let clipboard: Box<dyn ClipboardManager> = Box::new(WindowsClipboard);
        Ok(BoxClipboardManager::from(clipboard))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(KeyringError::UnsupportedPlatform)
    }
}

impl<T: ClipboardManager> ClipboardService<T> {
    pub fn copy_password(&mut self, password: &str) -> Result<(), KeyringError> {
        if !self.manager.is_supported() {
            return Err(KeyringError::ClipboardNotSupported);
        }

        if password.len() > self.config.max_content_length {
            return Err(KeyringError::InvalidInput {
                context: format!("Content exceeds maximum length of {}", self.config.max_content_length),
            });
        }

        self.manager.set_content(password)?;

        if self.config.clear_after_copy {
            let timeout = self.manager.timeout();
            std::thread::spawn(move || {
                std::thread::sleep(timeout);
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
