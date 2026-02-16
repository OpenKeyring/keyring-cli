//! 安全字符串类型
//!
//! 占位符模块，完整实现将在 Task 0.4 中完成。

use std::ops::Deref;

/// 安全字符串
///
/// 在内存中安全存储敏感数据，使用完后自动清零。
#[derive(Debug, Clone, Default)]
pub struct SecureString {
    _inner: String,
}

impl SecureString {
    /// 创建新的安全字符串
    #[must_use]
    pub fn new(s: String) -> Self {
        Self { _inner: s }
    }

    /// 获取字符串引用
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self._inner
    }

    /// 清除内容
    pub fn clear(&mut self) {
        self._inner.clear();
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// 敏感度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sensitivity {
    /// 非敏感
    #[default]
    None,
    /// 低敏感度
    Low,
    /// 中敏感度
    Medium,
    /// 高敏感度
    High,
}

/// 密码字段
#[derive(Debug, Clone, Default)]
pub struct PasswordField {
    pub _content: SecureString,
    pub _masked: bool,
}
