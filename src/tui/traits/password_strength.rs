//! 密码强度 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.4 中完成。

/// 密码强度等级
///
/// 表示密码的安全强度，从弱到强分为四个等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PasswordStrength {
    /// 弱密码：容易被猜测或破解
    Weak,
    /// 一般密码：有一定安全性
    #[default]
    Fair,
    /// 强密码：安全性较高
    Strong,
    /// 非常强：极高的安全性
    VeryStrong,
}

impl PasswordStrength {
    /// 获取强度分值 (0-100)
    #[must_use]
    pub const fn score(&self) -> u8 {
        match self {
            Self::Weak => 25,
            Self::Fair => 50,
            Self::Strong => 75,
            Self::VeryStrong => 100,
        }
    }

    /// 获取强度描述文本
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Weak => "弱",
            Self::Fair => "一般",
            Self::Strong => "强",
            Self::VeryStrong => "非常强",
        }
    }
}

impl std::fmt::Display for PasswordStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 密码强度计算器 trait
pub trait PasswordStrengthCalculator: Send + Sync {
    /// 计算密码强度
    fn calculate(&self, password: &str) -> PasswordStrength;

    /// 获取强度评分
    fn score(&self, password: &str) -> u8 {
        self.calculate(password).score()
    }
}
