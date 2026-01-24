pub mod manager;
pub mod macos;
pub mod linux;
pub mod windows;

pub use manager::ClipboardManager;
pub use macos::MacOSClipboard;
pub use linux::LinuxClipboard;
pub use windows::WindowsClipboard;