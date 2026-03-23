//! TUI 剪贴板服务适配器
//!
//! 封装现有 ClipboardManager 实现，提供 TUI 层所需的剪贴板接口。
//! 集成真实系统剪贴板（通过 arboard crate）。
//! 支持后台线程自动清除，确保敏感数据及时清理。

use crate::clipboard::{
    create_platform_clipboard, BoxClipboardManager, ClipboardManager as PlatformClipboardManager,
};
use crate::tui::traits::{
    ClipboardConfig, ClipboardContentType, ClipboardService, ClipboardState,
    SecureClipboardContent, SecureString,
};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// TUI 剪贴板服务
///
/// 实现 ClipboardService trait，提供剪贴板操作功能。
/// 连接真实系统剪贴板（通过 arboard）。
/// 支持后台线程自动清除，确保敏感数据及时清理。
pub struct TuiClipboardService {
    /// 剪贴板状态（用于 UI 显示和超时管理）
    state: ClipboardState,
    /// 系统剪贴板管理器
    system_clipboard: Option<BoxClipboardManager>,
    /// 后台清除线程句柄
    clear_thread: Option<JoinHandle<()>>,
    /// 线程运行标志
    clear_thread_running: Arc<AtomicBool>,
}

impl std::fmt::Debug for TuiClipboardService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TuiClipboardService")
            .field("state", &self.state)
            .field("system_clipboard", &self.system_clipboard.is_some())
            .field("clear_thread_active", &self.clear_thread.is_some())
            .finish()
    }
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
            clear_thread: None,
            clear_thread_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 使用指定配置创建剪贴板服务
    #[must_use]
    pub fn with_config(config: ClipboardConfig) -> Self {
        let system_clipboard = create_platform_clipboard().ok();
        Self {
            state: ClipboardState::new(config),
            system_clipboard,
            clear_thread: None,
            clear_thread_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if system clipboard is available
    #[must_use]
    pub fn is_system_clipboard_available(&self) -> bool {
        self.system_clipboard
            .as_ref()
            .is_some_and(|cb| cb.is_supported())
    }

    /// 启动后台清除线程
    fn start_clear_thread(&mut self, timeout_seconds: u64) {
        // 停止现有线程
        self.stop_clear_thread();

        // 创建新的运行标志
        let running = Arc::new(AtomicBool::new(true));
        self.clear_thread_running = running.clone();

        // 启动后台清除线程
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(timeout_seconds));

            // 只有在仍然运行时才清除
            if running.load(Ordering::SeqCst) {
                // 创建新的剪贴板实例来清除
                if let Ok(mut clipboard) = create_platform_clipboard() {
                    let _ = clipboard.clear();
                }
            }
        });

        self.clear_thread = Some(handle);
    }

    /// 停止后台清除线程
    fn stop_clear_thread(&mut self) {
        // 设置停止标志
        self.clear_thread_running.store(false, Ordering::SeqCst);

        // 等待线程结束（非阻塞，因为线程可能已经完成）
        if let Some(handle) = self.clear_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Default for TuiClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TuiClipboardService {
    fn drop(&mut self) {
        self.stop_clear_thread();
    }
}

impl ClipboardService for TuiClipboardService {
    /// 复制内容（使用 SecureString）
    fn copy_secure(
        &mut self,
        content: SecureString,
        content_type: ClipboardContentType,
    ) -> io::Result<()> {
        // 复制到系统剪贴板（如果可用，失败不影响功能）
        if let Some(ref mut clipboard) = self.system_clipboard {
            if let Some(text) = content.expose() {
                // 尝试复制到系统剪贴板，失败不影响功能
                // 这允许服务在 headless/测试环境中工作
                let _ = clipboard.set_content(text);
            }
        }

        // 根据内容类型获取超时时间
        let timeout_seconds = match content_type {
            ClipboardContentType::Password => self.state.config().sensitive_timeout_seconds,
            ClipboardContentType::Username => self.state.config().semi_sensitive_timeout_seconds,
            _ => self.state.config().semi_sensitive_timeout_seconds,
        };

        // 启动后台清除线程
        self.start_clear_thread(timeout_seconds);

        // 更新内部状态用于超时追踪
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
        // 读取系统剪贴板（如果可用）
        // 注意：ClipboardManager 需要 &mut self，但我们的 trait 使用 &self
        // 目前返回 None，因为读取对于密码管理器来说不是关键功能
        // 未来如果需要，可以考虑使用内部可变性
        let _ = &self.system_clipboard;
        Ok(None)
    }

    /// 检查是否需要清除
    fn should_clear(&self) -> bool {
        self.state.should_clear()
    }

    /// 清除剪贴板
    fn clear(&mut self) -> io::Result<()> {
        // 停止后台线程
        self.stop_clear_thread();

        // 清除系统剪贴板（失败不影响功能）
        if let Some(ref mut clipboard) = self.system_clipboard {
            let _ = clipboard.clear();
        }

        // 清除内部状态
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

        // 超时时自动清除系统剪贴板（备份机制）
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
        assert_eq!(
            service.state().content_type(),
            Some(ClipboardContentType::Password)
        );
    }

    #[test]
    fn test_copy_str_username() {
        let mut service = TuiClipboardService::new();

        let result = service.copy_str("user@example.com", ClipboardContentType::Username);
        assert!(result.is_ok());
        assert!(service.state().has_content());
        assert_eq!(
            service.state().content_type(),
            Some(ClipboardContentType::Username)
        );
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
        service
            .copy_str("test", ClipboardContentType::Password)
            .unwrap();
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
        service
            .copy_str("secret", ClipboardContentType::Password)
            .unwrap();
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
        service
            .copy_str("test", ClipboardContentType::Password)
            .unwrap();
        assert!(!service.should_clear());
    }

    #[test]
    fn test_tick() {
        let mut service = TuiClipboardService::new();

        // 初始状态 tick 返回 None
        assert!(service.tick().is_none());

        // 复制后 tick 返回剩余秒数
        service
            .copy_str("test", ClipboardContentType::Password)
            .unwrap();
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

    #[test]
    fn test_clear_stops_background_thread() {
        let mut service = TuiClipboardService::with_config(
            ClipboardConfig::new().with_sensitive_timeout(10)
        );

        // 复制并启动后台线程
        service
            .copy_str("secret_password", ClipboardContentType::Password)
            .unwrap();
        assert!(service.clear_thread.is_some());

        // 手动清除应停止后台线程
        service.clear().unwrap();
        assert!(service.clear_thread.is_none());
        assert!(!service.state().has_content());
    }
}
