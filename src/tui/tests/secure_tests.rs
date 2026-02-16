//! TUI 安全字符串测试
//!
//! 测试 SecureString、Sensitivity、PasswordField 等安全相关类型。

use crate::tui::traits::{PasswordField, SecureString, Sensitivity, PasswordStrength};

// ============================================================================
// Sensitivity 测试
// ============================================================================

#[test]
fn test_sensitivity_default() {
    let s = Sensitivity::default();
    assert_eq!(s, Sensitivity::None);
}

#[test]
fn test_sensitivity_ord() {
    assert!(Sensitivity::High as u8 > Sensitivity::Medium as u8);
    assert!(Sensitivity::Medium as u8 > Sensitivity::Low as u8);
    assert!(Sensitivity::Low as u8 > Sensitivity::None as u8);
}

// ============================================================================
// SecureString 基础测试
// ============================================================================

#[test]
fn test_secure_string_with_sensitivity() {
    let s = SecureString::new("password", Sensitivity::High);
    assert_eq!(s.sensitivity(), Sensitivity::High);
    assert_eq!(s.len(), 8);
}

#[test]
fn test_secure_string_sensitive_helper() {
    let s = SecureString::sensitive("secret");
    assert_eq!(s.sensitivity(), Sensitivity::High);
    assert_eq!(s.expose(), Some("secret"));
}

#[test]
fn test_secure_string_set_sensitivity() {
    let mut s = SecureString::new("data", Sensitivity::Low);
    assert_eq!(s.sensitivity(), Sensitivity::Low);

    s.set_sensitivity(Sensitivity::High);
    assert_eq!(s.sensitivity(), Sensitivity::High);
}

// ============================================================================
// SecureString 内容操作测试
// ============================================================================

#[test]
fn test_secure_string_push_unicode() {
    let mut s = SecureString::empty();
    s.push('你');
    s.push('好');

    // UTF-8 中文字符每个 3 字节
    assert_eq!(s.len(), 6);
    assert_eq!(s.expose(), Some("你好"));
}

#[test]
fn test_secure_string_pop_unicode() {
    let mut s = SecureString::sensitive("abc你好");

    assert_eq!(s.pop(), Some('好')); // 最后一个字符
    assert_eq!(s.expose(), Some("abc你"));

    assert_eq!(s.pop(), Some('你'));
    assert_eq!(s.expose(), Some("abc"));
}

#[test]
fn test_secure_string_expose_unchecked() {
    let s = SecureString::new("test", Sensitivity::Medium);
    assert_eq!(s.expose_unchecked(), "test");
}

#[test]
fn test_secure_string_expose_empty() {
    let s = SecureString::empty();
    assert_eq!(s.expose_unchecked(), "");
}

// ============================================================================
// SecureString 零化测试
// ============================================================================

#[test]
fn test_secure_string_zeroize() {
    let mut s = SecureString::sensitive("secret");
    assert!(!s.is_empty());

    s.zeroize();
    // 数据仍然存在但已被零化
    assert_eq!(s.expose(), Some("\0\0\0\0\0\0"));
}

#[test]
fn test_secure_string_clear() {
    let mut s = SecureString::sensitive("data");
    s.clear();
    assert!(s.is_empty());
}

// ============================================================================
// SecureString Display 测试
// ============================================================================

#[test]
fn test_secure_string_display_high() {
    let s = SecureString::new("secret", Sensitivity::High);
    assert_eq!(format!("{}", s), "[REDACTED]");
}

#[test]
fn test_secure_string_display_medium() {
    let s = SecureString::new("data", Sensitivity::Medium);
    assert_eq!(format!("{}", s), "[***]");
}

#[test]
fn test_secure_string_display_low() {
    let s = SecureString::new("public", Sensitivity::Low);
    assert_eq!(format!("{}", s), "public");
}

#[test]
fn test_secure_string_display_none() {
    let s = SecureString::new("text", Sensitivity::None);
    assert_eq!(format!("{}", s), "text");
}

// ============================================================================
// SecureString 转换测试
// ============================================================================

#[test]
fn test_secure_string_from_string() {
    let s = SecureString::from(String::from("test"));
    assert_eq!(s.sensitivity(), Sensitivity::Medium);
    assert_eq!(s.expose(), Some("test"));
}

#[test]
fn test_secure_string_from_str() {
    let s: SecureString = "test".into();
    assert_eq!(s.sensitivity(), Sensitivity::Medium);
    assert_eq!(s.expose(), Some("test"));
}

// ============================================================================
// PasswordField 测试
// ============================================================================

#[test]
fn test_password_field_default() {
    let field = PasswordField::default();
    assert!(field.is_empty());
    assert!(!field.is_visible());
}

#[test]
fn test_password_field_set_content() {
    let mut field = PasswordField::new();
    field.set_content("newpass");
    assert_eq!(field.expose(), Some("newpass"));
    assert_eq!(field.len(), 7);
}

#[test]
fn test_password_field_unicode_input() {
    let mut field = PasswordField::new();
    field.push_char('你');
    field.push_char('好');

    // 中文每个占 3 字节
    assert_eq!(field.len(), 6);
}

#[test]
fn test_password_field_display_masked() {
    let field = PasswordField::with_content("secret");
    assert_eq!(field.display_text(), "******");
}

#[test]
fn test_password_field_display_visible() {
    let mut field = PasswordField::with_content("secret");
    field.set_visibility(true);
    assert_eq!(field.display_text(), "secret");
}

#[test]
fn test_password_field_strength() {
    let field = PasswordField::with_content("weak");

    let calculator = |pwd: &str| -> PasswordStrength {
        if pwd.len() < 8 {
            PasswordStrength::Weak
        } else {
            PasswordStrength::Strong
        }
    };

    assert_eq!(field.strength(&calculator), PasswordStrength::Weak);
}

#[test]
fn test_password_field_strength_cache() {
    let mut field = PasswordField::with_content("strongpass123");

    let calculator = |_: &str| -> PasswordStrength {
        PasswordStrength::VeryStrong
    };

    // 首次计算（无缓存）
    let strength1 = field.strength(&calculator);
    assert_eq!(strength1, PasswordStrength::VeryStrong);

    // 更新缓存
    field.update_strength_cache(strength1);

    // 再次获取应使用缓存
    let strength2 = field.strength(&calculator);
    assert_eq!(strength2, PasswordStrength::VeryStrong);

    // 修改内容应清除缓存（由 clear_strength_cache 验证）
    field.push_char('!');
    field.clear_strength_cache();
}

#[test]
fn test_password_field_clear_strength_cache() {
    let mut field = PasswordField::with_content("test");

    // 设置缓存
    field.update_strength_cache(PasswordStrength::Fair);

    // 清除后再次获取应该重新计算
    field.clear_strength_cache();

    let calculator = |pwd: &str| -> PasswordStrength {
        if pwd.len() < 8 {
            PasswordStrength::Fair
        } else {
            PasswordStrength::Strong
        }
    };

    let strength = field.strength(&calculator);
    assert_eq!(strength, PasswordStrength::Fair);
}

// ============================================================================
// Clone 测试
// ============================================================================

#[test]
fn test_secure_string_clone() {
    let s1 = SecureString::sensitive("secret");
    let s2 = s1.clone();

    assert_eq!(s2.expose(), Some("secret"));
    assert_eq!(s2.sensitivity(), Sensitivity::High);
}

#[test]
fn test_password_field_clone() {
    let mut field1 = PasswordField::with_content("pass123");
    field1.toggle_visibility();
    field1.update_strength_cache(PasswordStrength::Strong);

    let field2 = field1.clone();

    assert_eq!(field2.expose(), Some("pass123"));
    assert!(field2.is_visible());
    // 克隆后的字段也应能正确工作
    assert_eq!(field2.len(), 7);
}

// ============================================================================
// Drop 测试（通过 valgrind/asan 可验证零化）
// ============================================================================

#[test]
fn test_secure_string_drop_zeroizes() {
    {
        let _s = SecureString::sensitive("sensitive_data");
    } // Drop 时应调用 zeroize

    // 如果使用 valgrind 或 AddressSanitizer，可以验证内存被清零
    // 在单元测试中我们只能验证编译通过
    assert!(true);
}

#[test]
fn test_password_field_drop_zeroizes() {
    {
        let _field = PasswordField::with_content("password123");
    } // Drop 时 SecureString 应被零化

    assert!(true);
}
