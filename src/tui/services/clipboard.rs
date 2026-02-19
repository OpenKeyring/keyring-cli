//! TUI 剪贴板服务适配器
//!
//! 封装现有 ClipboardManager 实现，提供 TUI 层所需的剪贴板接口。

use crate::tui::traits::{
    ClipboardConfig, ClipboardContentType, ClipboardService, ClipboardState, SecureClipboardContent,
    SecureString,
};
use std::io;

/// TUI 剪贴板服务
///
/// 实现 ClipboardService trait，提供剪贴板操作功能。
pub struct TuiClipboardService {
    /// 剪贴板状态
    state: ClipboardState,
}

impl TuiClipboardService {
    /// 创建新的剪贴板服务
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ClipboardState::new(ClipboardConfig::default()),
        }
    }

    /// 使用指定配置创建剪贴板服务
    #[must_use]
    pub fn with_config(config: ClipboardConfig) -> Self {
        Self {
            state: ClipboardState::new(config),
        }
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
        // TODO: 调用系统剪贴板 API
        // 当前仅更新内部状态
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
        // TODO: 调用系统剪贴板 API
        // 当前返回 None，因为实际剪贴板读取需要系统 API
        Ok(None)
    }

    /// 检查是否需要清除
    fn should_clear(&self) -> bool {
        self.state.should_clear()
    }

    /// 清除剪贴板
    fn clear(&mut self) -> io::Result<()> {
        // TODO: 调用系统剪贴板 API 清除系统剪贴板
        self.state.clear_sensitive();
        Ok(())
    }

    /// 获取当前状态
    fn state(&self) -> &ClipboardState {
        &self.state
    }

    /// 更新倒计时（每秒调用）
    fn tick(&mut self) -> Option<u64> {
        self.state.tick()
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
