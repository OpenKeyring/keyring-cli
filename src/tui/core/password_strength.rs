//! 默认密码强度计算器实现
//!
//! 占位符模块，完整实现将在 Task C.7 中完成。

use crate::tui::traits::{PasswordStrength, PasswordStrengthCalculator};

/// 默认密码强度计算器
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPasswordStrengthCalculator;

impl DefaultPasswordStrengthCalculator {
    /// 创建新的计算器
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 根据长度计算基础强度
    fn calculate_by_length(&self, password: &str) -> PasswordStrength {
        let len = password.chars().count();
        match len {
            0..=7 => PasswordStrength::Weak,
            8..=11 => PasswordStrength::Fair,
            12..=15 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }
}

impl PasswordStrengthCalculator for DefaultPasswordStrengthCalculator {
    fn calculate(&self, password: &str) -> PasswordStrength {
        self.calculate_by_length(password)
    }
}
