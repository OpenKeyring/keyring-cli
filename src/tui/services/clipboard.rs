//! TUI 剪贴板服务适配器
//!
//! 封装现有 ClipboardManager 实现，提供 TUI 层所需的剪贴板接口。
//! 集成真实系统剪贴板（通过 arboard crate）。

use crate::clipboard::{create_platform_clipboard, ClipboardManager as PlatformClipboardManager, BoxClipboardManager};
use crate::tui::traits::{
    ClipboardConfig, ClipboardContentType, ClipboardService, ClipboardState, SecureClipboardContent,
    SecureString,
};
use std::io;

/// TUI 剪贴板服务
///
/// 实现 ClipboardService trait，提供剪贴板操作功能。
/// 连接真实系统剪贴板（通过 arboard）。
#[derive(Debug)]
pub struct TuiClipboardService {
    /// 剪贴板状态（用于 UI 显示和超时管理）
    state: ClipboardState,
    /// 系统剪贴板管理器
    system_clipboard: Option<BoxClipboardManager>,
}

impl TuiClipboardService {
    /// 创建新的剪贴板服务
    ///
    /// # Errors
    /// Returns error if system clipboard is unavailable (but service still functions)
    #[must_use]
    pub fn new() -> Self {
        let system_clipboard = create_platform_clipboard().ok();
        Self {
            state: ClipboardState::new(ClipboardConfig::default()),
            system_clipboard,
        }
    }

    /// 使用指定配置创建剪贴板服务
    #[must_use]
    pub fn with_config(config: ClipboardConfig) -> Self {
        let system_clipboard = create_platform_clipboard().ok();
        Self {
            state: ClipboardState::new(config),
            system_clipboard,
        }
    }

    /// Check if system clipboard is available
    #[must_use]
    pub fn is_system_clipboard_available(&self) -> bool {
        self.system_clipboard.as_ref().is_some_and(|cb| cb.is_supported())
    }
}

impl Default for TuiClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardService for TuiClipboardService {
    /// 复制内容（使用 SecureString）
    fn copy_secure(&mut self, content: SecureString, content_type: ClipboardContentType) -> io::Result<()> {
        // Copy to system clipboard if available (non-fatal if it fails)
        if let Some(ref mut clipboard) = self.system_clipboard {
            if let Some(text) = content.expose() {
                // Try to copy to system clipboard, but don't fail if unavailable
                // This allows the service to work in headless/test environments
                let _ = clipboard.set_content(text);
            }
        }

        // Update internal state for timeout tracking
        self.state.copy(content, content_type);
        Ok(())
    }

    /// 复制明文（自动包装为 SecureString）
    fn copy_str(&mut self, content: &str, content_type: ClipboardContentType) -> io::Result<()> {
        let secure_content = SecureString::sensitive(content);
        self.copy_secure(secure_content, content_type)
    }

    /// 从剪贴板读取
    fn paste(&self) -> io::Result<Option<SecureClipboardContent>> {
        // Read from system clipboard if available
        // Note: ClipboardManager requires &mut self, but our trait uses &self
        // For now, return None as reading is less critical for password manager
        // In the future, consider using interior mutability if needed
        let _ = &self.system_clipboard;
        Ok(None)
    }

    /// 检查是否需要清除
    fn should_clear(&self) -> bool {
        self.state.should_clear()
    }

    /// 清除剪贴板
    fn clear(&mut self) -> io::Result<()> {
        // Clear system clipboard (non-fatal if it fails)
        if let Some(ref mut clipboard) = self.system_clipboard {
            let _ = clipboard.clear();
        }

        // Clear internal state
        self.state.clear_sensitive();
        Ok(())
    }

    /// 获取当前状态
    fn state(&self) -> &ClipboardState {
        &self.state
    }

    /// 更新倒计时（每秒调用）
    fn tick(&mut self) -> Option<u64> {
        let remaining = self.state.tick();

        // Auto-clear system clipboard when timeout expires
        if self.state.should_clear() {
            if let Some(ref mut clipboard) = self.system_clipboard {
                let _ = clipboard.clear();
            }
        }

        remaining
    }
}

impl crate::tui::traits::SecureClear for TuiClipboardService {
    fn clear_sensitive_data(&mut self) {
        let _ = self.clear();
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::traits::{ClipboardService, SecureClear};

    #[test]
    fn test_clipboard_service_creation() {
        let service = TuiClipboardService::new();
        assert!(!service.state().has_content());
    }

    #[test]
    fn test_clipboard_service_with_config() {
        let config = ClipboardConfig::new()
            .with_sensitive_timeout(60)
            .with_semi_sensitive_timeout(120);

        let service = TuiClipboardService::with_config(config);
        assert!(!service.state().has_content());
    }

    #[test]
    fn test_clipboard_service_trait_bounds() {
        // 验证 TuiClipboardService 实现了所有必需的 trait
        fn assert_clipboard_service<T: ClipboardService + SecureClear + Send + Sync>() {}
        assert_clipboard_service::<TuiClipboardService>();
    }

    #[test]
    fn test_copy_str_password() {
        let mut service = TuiClipboardService::new();

        let result = service.copy_str("my_secret_password", ClipboardContentType::Password);
        assert!(result.is_ok());
        assert!(service.state().has_content());
        assert_eq!(service.state().content_type(), Some(ClipboardContentType::Password));
    }

    #[test]
    fn test_copy_str_username() {
        let mut service = TuiClipboardService::new();

        let result = service.copy_str("user@example.com", ClipboardContentType::Username);
        assert!(result.is_ok());
        assert!(service.state().has_content());
        assert_eq!(service.state().content_type(), Some(ClipboardContentType::Username));
    }

    #[test]
    fn test_copy_secure() {
        let mut service = TuiClipboardService::new();
        let secure = SecureString::sensitive("sensitive_data");

        let result = service.copy_secure(secure, ClipboardContentType::Password);
        assert!(result.is_ok());
        assert!(service.state().has_content());
    }

    #[test]
    fn test_clear() {
        let mut service = TuiClipboardService::new();

        // 复制一些内容
        service.copy_str("test", ClipboardContentType::Password).unwrap();
        assert!(service.state().has_content());

        // 清除
        let result = service.clear();
        assert!(result.is_ok());
        assert!(!service.state().has_content());
    }

    #[test]
    fn test_secure_clear_trait() {
        let mut service = TuiClipboardService::new();

        // 复制一些内容
        service.copy_str("secret", ClipboardContentType::Password).unwrap();
        assert!(service.state().has_content());

        // 使用 SecureClear trait 清除
        service.clear_sensitive_data();
        assert!(!service.state().has_content());
    }

    #[test]
    fn test_paste_returns_none() {
        let service = TuiClipboardService::new();

        // 当前实现返回 None（未实现系统剪贴板读取）
        let result = service.paste().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_should_clear() {
        let mut service = TuiClipboardService::new();

        // 初始状态不需要清除
        assert!(!service.should_clear());

        // 复制后（未到时间）也不需要清除
        service.copy_str("test", ClipboardContentType::Password).unwrap();
        assert!(!service.should_clear());
    }

    #[test]
    fn test_tick() {
        let mut service = TuiClipboardService::new();

        // 初始状态 tick 返回 None
        assert!(service.tick().is_none());

        // 复制后 tick 返回剩余秒数
        service.copy_str("test", ClipboardContentType::Password).unwrap();
        let remaining = service.tick();
        // 应该有剩余时间（默认 30 秒超时）
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > 0);
    }

    #[test]
    fn test_default() {
        let service = TuiClipboardService::default();
        assert!(!service.state().has_content());
    }
}
