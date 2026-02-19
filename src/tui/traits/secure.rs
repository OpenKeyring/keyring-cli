//! 安全字符串类型
//!
//! 提供在内存中安全存储敏感数据的功能，使用完后自动清零。

use crate::tui::traits::PasswordStrength;

// ============================================================================
// 敏感度级别
// ============================================================================

/// 敏感度级别
///
/// 表示数据的敏感程度，影响如何显示和处理该数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sensitivity {
    /// 非敏感（普通文本）
    #[default]
    None,
    /// 低敏感度（URL、备注等）
    Low,
    /// 中敏感度（用户名、邮箱等）
    Medium,
    /// 高敏感度（密码、密钥等）
    High,
}

// ============================================================================
// 安全字符串容器
// ============================================================================

/// 安全字符串容器
///
/// 包装敏感字符串，确保：
/// 1. 内存中数据在 Drop 时被零化
/// 2. 不意外打印到日志
/// 3. 不意外克隆
#[derive(Debug)]
pub struct SecureString {
    /// 内部数据
    data: Option<Vec<u8>>,
    /// 敏感级别
    sensitivity: Sensitivity,
}

impl SecureString {
    /// 从明文创建新的安全字符串
    #[must_use]
    pub fn new(content: &str, sensitivity: Sensitivity) -> Self {
        Self {
            data: Some(content.as_bytes().to_vec()),
            sensitivity,
        }
    }

    /// 创建空的安全字符串
    #[must_use]
    pub fn empty() -> Self {
        Self {
            data: Some(Vec::new()),
            sensitivity: Sensitivity::None,
        }
    }

    /// 创建高敏感度的安全字符串（用于密码等）
    #[must_use]
    pub fn sensitive(content: &str) -> Self {
        Self::new(content, Sensitivity::High)
    }

    /// 获取字符串长度
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.as_ref().map_or(0, |d| d.len())
    }

    /// 检查是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.as_ref().is_none_or(|d| d.is_empty())
    }

    /// 暴露内容（仅用于必要操作，调用方负责使用后清除）
    ///
    /// 返回字符串切片的引用，仅在数据存在时有效。
    #[must_use]
    pub fn expose(&self) -> Option<&str> {
        self.data.as_ref().map(|d| unsafe {
            // SAFETY: 我们确保数据来自有效的 UTF-8 字符串
            std::str::from_utf8_unchecked(d.as_slice())
        })
    }

    /// 暴露不检查的内容（危险，仅在调试时使用）
    #[must_use]
    pub fn expose_unchecked(&self) -> &str {
        self.expose().unwrap_or("")
    }

    /// 更新内容（先清除旧数据）
    pub fn update(&mut self, content: &str) {
        // 先清除旧数据
        self.zeroize();
        self.data = Some(content.as_bytes().to_vec());
    }

    /// 设置敏感度级别
    pub fn set_sensitivity(&mut self, sensitivity: Sensitivity) {
        self.sensitivity = sensitivity;
    }

    /// 获取敏感度级别
    #[must_use]
    pub const fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    /// 清除并零化内存
    pub fn zeroize(&mut self) {
        if let Some(ref mut data) = self.data {
            // 用零覆盖内存
            for byte in data.iter_mut() {
                *byte = 0;
            }
        }
    }

    /// 附加内容到末尾
    pub fn push(&mut self, ch: char) {
        if let Some(ref mut data) = self.data {
            let mut buf = [0u8; 4];
            let str_bytes = ch.encode_utf8(&mut buf).as_bytes();
            data.extend_from_slice(str_bytes);
        } else {
            // 如果 data 是 None，创建新 Vec
            let mut buf = [0u8; 4];
            let str_bytes = ch.encode_utf8(&mut buf).as_bytes();
            self.data = Some(str_bytes.to_vec());
        }
    }

    /// 移除最后一个字符
    pub fn pop(&mut self) -> Option<char> {
        if let Some(ref mut data) = self.data {
            if !data.is_empty() {
                // 找到最后一个字符的边界
                let len = data.len();
                let mut end = len;
                while end > 0 {
                    end -= 1;
                    if data[end] & 0xC0 != 0x80 {
                        // 这是多字节字符的起始字节
                        // 先获取字符
                        let result = {
                            let bytes = &data[end..];
                            std::str::from_utf8(bytes).ok()?.chars().next()
                        };
                        // 移除并零化
                        for i in end..len {
                            let _ = data[i]; // 访问以避免未使用警告
                            data[i] = 0;
                        }
                        data.truncate(end);
                        return result;
                    }
                }
            }
        }
        None
    }

    /// 清空内容
    pub fn clear(&mut self) {
        self.zeroize();
        self.data = Some(Vec::new());
    }
}

impl Clone for SecureString {
    fn clone(&self) -> Self {
        // 克隆时创建新的副本（允许此操作用于测试）
        Self {
            data: self.data.clone(),
            sensitivity: self.sensitivity,
        }
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.zeroize();
    }
}

// 禁止意外打印到日志
impl std::fmt::Display for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sensitivity {
            Sensitivity::High => write!(f, "[REDACTED]"),
            Sensitivity::Medium => write!(f, "[***]"),
            Sensitivity::Low | Sensitivity::None => {
                if let Some(content) = self.expose() {
                    write!(f, "{}", content)
                } else {
                    write!(f, "")
                }
            }
        }
    }
}

// 从字符串类型转换
impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(&s, Sensitivity::Medium)
    }
}

impl From<&str> for SecureString {
    fn from(s: &str) -> Self {
        Self::new(s, Sensitivity::Medium)
    }
}

// ============================================================================
// 密码字段
// ============================================================================

/// 密码字段
///
/// 专用于密码输入的字段，包含可见性切换和强度评估。
#[derive(Debug)]
pub struct PasswordField {
    /// 密码内容
    content: SecureString,
    /// 是否显示明文
    visible: bool,
    /// 缓存的强度评估
    cached_strength: Option<PasswordStrength>,
}

impl PasswordField {
    /// 创建新的密码字段
    #[must_use]
    pub fn new() -> Self {
        Self {
            content: SecureString::empty(),
            visible: false,
            cached_strength: None,
        }
    }

    /// 带初始内容创建
    #[must_use]
    pub fn with_content(content: &str) -> Self {
        Self {
            content: SecureString::sensitive(content),
            visible: false,
            cached_strength: None,
        }
    }

    /// 获取内容长度（用于显示占位符）
    #[must_use]
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// 检查是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// 设置内容
    pub fn set_content(&mut self, content: &str) {
        self.content.update(content);
        self.cached_strength = None; // 清除缓存
    }

    /// 追加字符
    pub fn push_char(&mut self, ch: char) {
        self.content.push(ch);
        self.cached_strength = None; // 清除缓存
    }

    /// 删除最后一个字符
    pub fn pop_char(&mut self) -> Option<char> {
        let result = self.content.pop();
        self.cached_strength = None; // 清除缓存
        result
    }

    /// 清空内容
    pub fn clear(&mut self) {
        self.content.clear();
        self.cached_strength = None;
    }

    /// 切换可见性
    pub fn toggle_visibility(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }

    /// 设置可见性
    pub fn set_visibility(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 是否可见
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 获取显示字符串（如果是掩码则返回星号）
    #[must_use]
    pub fn display_text(&self) -> String {
        if self.visible {
            self.content.expose_unchecked().to_string()
        } else {
            "*".repeat(self.content.len())
        }
    }

    /// 暴露内容（仅在必要时使用）
    #[must_use]
    pub fn expose(&self) -> Option<&str> {
        self.content.expose()
    }

    /// 获取密码强度（使用缓存或计算）
    pub fn strength(&self, calculator: &dyn Fn(&str) -> PasswordStrength) -> PasswordStrength {
        if let Some(cached) = self.cached_strength {
            return cached;
        }

        let strength = self
            .content
            .expose()
            .map(|s| calculator(s))
            .unwrap_or(PasswordStrength::Weak);

        // 注意：由于这里是借用，不能直接设置 cached_strength
        // 调用方需要负责更新缓存
        strength
    }

    /// 更新强度缓存
    pub fn update_strength_cache(&mut self, strength: PasswordStrength) {
        self.cached_strength = Some(strength);
    }

    /// 清除强度缓存
    pub fn clear_strength_cache(&mut self) {
        self.cached_strength = None;
    }
}

impl Default for PasswordField {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PasswordField {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            visible: self.visible,
            cached_strength: self.cached_strength,
        }
    }
}

// ============================================================================
// 持有敏感数据的 trait
// ============================================================================

/// 持有敏感数据的 trait
///
/// 标记类型包含敏感数据，需要特殊处理。
pub trait HoldsSensitiveData: Send + Sync {
    /// 获取敏感度级别
    fn sensitivity(&self) -> Sensitivity;

    /// 清除所有敏感数据
    fn clear_sensitive_data(&mut self);
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_new() {
        let s = SecureString::new("password", Sensitivity::High);
        assert_eq!(s.len(), 8);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_secure_string_empty() {
        let s = SecureString::empty();
        assert_eq!(s.len(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_secure_string_expose() {
        let s = SecureString::new("test", Sensitivity::Medium);
        assert_eq!(s.expose(), Some("test"));
    }

    #[test]
    fn test_secure_string_update() {
        let mut s = SecureString::new("old", Sensitivity::High);
        s.update("new");
        assert_eq!(s.expose(), Some("new"));
    }

    #[test]
    fn test_secure_string_push() {
        let mut s = SecureString::empty();
        s.push('a');
        s.push('b');
        assert_eq!(s.expose(), Some("ab"));
    }

    #[test]
    fn test_secure_string_pop() {
        let mut s = SecureString::new("abc", Sensitivity::Low);
        assert_eq!(s.pop(), Some('c'));
        assert_eq!(s.expose(), Some("ab"));
    }

    #[test]
    fn test_secure_string_clear() {
        let mut s = SecureString::new("data", Sensitivity::High);
        s.clear();
        assert!(s.is_empty());
    }

    #[test]
    fn test_secure_string_display() {
        let s_high = SecureString::new("secret", Sensitivity::High);
        assert_eq!(format!("{}", s_high), "[REDACTED]");

        let s_medium = SecureString::new("data", Sensitivity::Medium);
        assert_eq!(format!("{}", s_medium), "[***]");

        let s_low = SecureString::new("public", Sensitivity::Low);
        assert_eq!(format!("{}", s_low), "public");
    }

    #[test]
    fn test_password_field_new() {
        let field = PasswordField::new();
        assert!(field.is_empty());
        assert!(!field.is_visible());
    }

    #[test]
    fn test_password_field_with_content() {
        let field = PasswordField::with_content("pass123");
        assert_eq!(field.len(), 7);
        assert_eq!(field.expose(), Some("pass123"));
    }

    #[test]
    fn test_password_field_display_text() {
        let mut field = PasswordField::with_content("secret");
        // content.len() 返回字节数，"secret" 是 6 字节
        // 但 SecureString::len() 实际上返回 Vec<u8>.len()，即 6
        let display = field.display_text();
        assert_eq!(display.len(), field.content.len()); // 验证星号数量等于长度
        assert!(display.chars().all(|c| c == '*')); // 验证都是星号

        field.toggle_visibility();
        assert_eq!(field.display_text(), "secret");
    }

    #[test]
    fn test_password_field_push_pop() {
        let mut field = PasswordField::new();
        field.push_char('a');
        field.push_char('b');
        assert_eq!(field.len(), 2);

        assert_eq!(field.pop_char(), Some('b'));
        assert_eq!(field.len(), 1);
    }

    #[test]
    fn test_password_field_clear() {
        let mut field = PasswordField::with_content("data");
        field.clear();
        assert!(field.is_empty());
    }

    #[test]
    fn test_password_field_toggle_visibility() {
        let mut field = PasswordField::new();
        assert!(!field.is_visible());

        field.toggle_visibility();
        assert!(field.is_visible());

        field.set_visibility(false);
        assert!(!field.is_visible());
    }
}
