//! Secure clipboard with auto-clear functionality

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod manager;
#[cfg(target_os = "windows")]
pub mod windows;

// use std::time::Duration;  // Unused

// Re-exports from manager module
pub use manager::{
    create_platform_clipboard, BoxClipboardManager, ClipboardConfig, ClipboardManager,
    ClipboardService,
};

// Platform-specific exports
#[cfg(target_os = "linux")]
pub use linux::LinuxClipboard;
#[cfg(target_os = "macos")]
pub use macos::MacOSClipboard;
#[cfg(target_os = "windows")]
pub use windows::WindowsClipboard;

/// Clipboard clearing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearMode {
    Clear,   // Empty clipboard
    Restore, // Restore original content
}

/// Platform-specific clipboard backend trait
pub trait ClipboardBackend: Send + Sync {
    fn set_text(&self, text: &str) -> anyhow::Result<()>;
    fn get_text(&self) -> anyhow::Result<String>;
    fn is_available(&self) -> bool;
}
