//! 剪贴板状态管理 trait 定义
//!
//! 定义 TUI 剪贴板服务相关的接口，包括敏感数据处理和自动清除功能。

use crate::tui::traits::secure::SecureString;
use std::time::{Duration, Instant};

// ============================================================================
// 剪贴板内容类型与敏感级别
// ============================================================================

/// 剪贴板内容敏感级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardSensitivity {
    /// 敏感内容（密码）- 强制自动清除
    Sensitive,
    /// 半敏感内容（用户名）- 可配置自动清除
    SemiSensitive,
    /// 普通内容（URL、备注）- 不自动清除
    Normal,
}

/// 剪贴板内容类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardContentType {
    Username,
    Password,
    Url,
    Notes,
    Other,
}

impl ClipboardContentType {
    /// 获取默认敏感级别
    #[must_use]
    pub const fn default_sensitivity(&self) -> ClipboardSensitivity {
        match self {
            Self::Password => ClipboardSensitivity::Sensitive,
            Self::Username => ClipboardSensitivity::SemiSensitive,
            Self::Url | Self::Notes | Self::Other => ClipboardSensitivity::Normal,
        }
    }
}

// ============================================================================
// 剪贴板配置
// ============================================================================

/// 剪贴板配置
#[derive(Debug, Clone)]
pub struct ClipboardConfig {
    /// 敏感内容清除超时（秒）
    pub sensitive_timeout_seconds: u64,
    /// 半敏感内容清除超时（秒），0 表示不清除
    pub semi_sensitive_timeout_seconds: u64,
    /// 是否清除半敏感内容
    pub clear_semi_sensitive: bool,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            sensitive_timeout_seconds: 30,
            semi_sensitive_timeout_seconds: 60,
            clear_semi_sensitive: false,
        }
    }
}

impl ClipboardConfig {
    /// 创建新的配置
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sensitive_timeout_seconds: 30,
            semi_sensitive_timeout_seconds: 60,
            clear_semi_sensitive: false,
        }
    }

    /// 设置敏感内容超时
    #[must_use]
    pub const fn with_sensitive_timeout(mut self, seconds: u64) -> Self {
        self.sensitive_timeout_seconds = seconds;
        self
    }

    /// 设置半敏感内容超时
    #[must_use]
    pub const fn with_semi_sensitive_timeout(mut self, seconds: u64) -> Self {
        self.semi_sensitive_timeout_seconds = seconds;
        self
    }

    /// 设置是否清除半敏感内容
    #[must_use]
    pub const fn with_clear_semi_sensitive(mut self, clear: bool) -> Self {
        self.clear_semi_sensitive = clear;
        self
    }
}

// ============================================================================
// 安全剪贴板内容
// ============================================================================

/// 安全剪贴板内容容器
///
/// 内部使用 SecureString 存储敏感内容
#[derive(Debug)]
pub struct SecureClipboardContent {
    /// 内容（使用 SecureString 保护）
    content: SecureString,
    /// 内容类型
    content_type: ClipboardContentType,
    /// 复制时间
    copied_at: Instant,
}

impl SecureClipboardContent {
    /// 创建新的剪贴板内容
    #[must_use]
    pub fn new(content: SecureString, content_type: ClipboardContentType) -> Self {
        Self {
            content,
            content_type,
            copied_at: Instant::now(),
        }
    }

    /// 获取内容（谨慎使用）
    pub fn expose(&self) -> Option<&str> {
        self.content.expose()
    }

    /// 获取内容类型
    #[must_use]
    pub const fn content_type(&self) -> ClipboardContentType {
        self.content_type
    }

    /// 获取敏感级别
    #[must_use]
    pub const fn sensitivity(&self) -> ClipboardSensitivity {
        self.content_type.default_sensitivity()
    }

    /// 获取已存在时间
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.copied_at.elapsed()
    }

    /// 清除内容
    pub fn clear(&mut self) {
        self.content.zeroize();
    }

    /// 是否应该自动清除
    #[must_use]
    pub fn should_auto_clear(&self, config: &ClipboardConfig) -> bool {
        match self.sensitivity() {
            ClipboardSensitivity::Sensitive => true,
            ClipboardSensitivity::SemiSensitive => config.clear_semi_sensitive,
            ClipboardSensitivity::Normal => false,
        }
    }
}

// ============================================================================
// 剪贴板状态
// ============================================================================

/// 剪贴板状态
#[derive(Debug)]
pub struct ClipboardState {
    /// 当前内容（使用安全容器）
    current: Option<SecureClipboardContent>,
    /// 配置
    config: ClipboardConfig,
    /// 倒计时剩余秒数（用于 UI 显示）
    remaining_seconds: Option<u64>,
}

impl ClipboardState {
    /// 创建新的剪贴板状态
    #[must_use]
    pub fn new(config: ClipboardConfig) -> Self {
        Self {
            current: None,
            config,
            remaining_seconds: None,
        }
    }

    /// 复制内容
    pub fn copy(&mut self, content: SecureString, content_type: ClipboardContentType) {
        self.current = Some(SecureClipboardContent::new(content, content_type));
        self.update_remaining();
    }

    /// 检查是否需要清除
    #[must_use]
    pub fn should_clear(&self) -> bool {
        self.current.as_ref().is_some_and(|c| {
            if !c.should_auto_clear(&self.config) {
                return false;
            }
            let timeout = match c.sensitivity() {
                ClipboardSensitivity::Sensitive => self.config.sensitive_timeout_seconds,
                ClipboardSensitivity::SemiSensitive => self.config.semi_sensitive_timeout_seconds,
                ClipboardSensitivity::Normal => return false,
            };
            c.elapsed().as_secs() >= timeout
        })
    }

    /// 清除敏感内容
    pub fn clear_sensitive(&mut self) {
        if let Some(ref mut content) = self.current {
            content.clear();
        }
        self.current = None;
        self.remaining_seconds = None;
    }

    /// 更新剩余时间
    fn update_remaining(&mut self) {
        self.remaining_seconds = self.current.as_ref().and_then(|c| {
            if !c.should_auto_clear(&self.config) {
                return None;
            }
            let timeout = match c.sensitivity() {
                ClipboardSensitivity::Sensitive => self.config.sensitive_timeout_seconds,
                ClipboardSensitivity::SemiSensitive => self.config.semi_sensitive_timeout_seconds,
                ClipboardSensitivity::Normal => return None,
            };
            let elapsed = c.elapsed().as_secs();
            if elapsed >= timeout {
                None
            } else {
                Some(timeout - elapsed)
            }
        });
    }

    /// 每秒调用，更新状态
    pub fn tick(&mut self) -> Option<u64> {
        self.update_remaining();
        if self.should_clear() {
            self.clear_sensitive();
        }
        self.remaining_seconds
    }

    /// 获取当前内容类型
    #[must_use]
    pub fn content_type(&self) -> Option<ClipboardContentType> {
        self.current.as_ref().map(|c| c.content_type())
    }

    /// 是否有内容
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.current.is_some()
    }
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self::new(ClipboardConfig::default())
    }
}

// ============================================================================
// 剪贴板服务 Trait
// ============================================================================

/// 剪贴板服务 trait
pub trait ClipboardService: Send + Sync {
    /// 复制内容（使用 SecureString）
    fn copy_secure(&mut self, content: SecureString, content_type: ClipboardContentType) -> std::io::Result<()>;

    /// 复制明文（自动包装为 SecureString）
    fn copy_str(&mut self, content: &str, content_type: ClipboardContentType) -> std::io::Result<()>;

    /// 从剪贴板读取
    fn paste(&self) -> std::io::Result<Option<SecureClipboardContent>>;

    /// 检查是否需要清除
    fn should_clear(&self) -> bool;

    /// 清除剪贴板
    fn clear(&mut self) -> std::io::Result<()>;

    /// 获取当前状态
    fn state(&self) -> &ClipboardState;

    /// 更新倒计时（每秒调用）
    fn tick(&mut self) -> Option<u64>;
}

/// 剪贴板内容（兼容旧代码）
#[derive(Debug, Clone, Default)]
pub struct ClipboardContent {
    pub text: Option<String>,
}

impl ClipboardContent {
    /// 创建新的剪贴板内容
    #[must_use]
    pub fn new(text: Option<String>) -> Self {
        Self { text }
    }

    /// 从字符串创建
    #[must_use]
    pub fn from_string(text: String) -> Self {
        Self { text: Some(text) }
    }

    /// 是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.as_ref().is_none_or(|s| s.is_empty())
    }
}
