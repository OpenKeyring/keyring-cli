//! 密码强度 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.4 中完成。

/// 密码强度等级（枚举形式，用于与主题系统集成）
///
/// 表示密码的安全强度，从非常弱到非常强分为五个等级。
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

/// 密码强度等级（详细分类）
///
/// 提供更细粒度的强度评估。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthLevel {
    /// 非常弱
    VeryWeak,
    /// 弱
    Weak,
    /// 一般
    Fair,
    /// 强
    Strong,
    /// 非常强
    VeryStrong,
}

impl Default for StrengthLevel {
    fn default() -> Self {
        Self::Fair
    }
}

impl StrengthLevel {
    /// 从分数 (0-100) 转换
    #[must_use]
    pub const fn from_score(score: u8) -> Self {
        match score {
            0..=20 => Self::VeryWeak,
            21..=40 => Self::Weak,
            41..=60 => Self::Fair,
            61..=80 => Self::Strong,
            _ => Self::VeryStrong,
        }
    }

    /// 获取显示文本
    #[must_use]
    pub const fn display_text(&self) -> &str {
        match self {
            Self::VeryWeak => "非常弱",
            Self::Weak => "弱",
            Self::Fair => "一般",
            Self::Strong => "强",
            Self::VeryStrong => "非常强",
        }
    }

    /// 获取对应分数
    #[must_use]
    pub const fn score(&self) -> u8 {
        match self {
            Self::VeryWeak => 10,
            Self::Weak => 30,
            Self::Fair => 50,
            Self::Strong => 75,
            Self::VeryStrong => 100,
        }
    }
}

impl From<PasswordStrength> for StrengthLevel {
    fn from(strength: PasswordStrength) -> Self {
        match strength {
            PasswordStrength::Weak => Self::Weak,
            PasswordStrength::Fair => Self::Fair,
            PasswordStrength::Strong => Self::Strong,
            PasswordStrength::VeryStrong => Self::VeryStrong,
        }
    }
}

impl From<StrengthLevel> for PasswordStrength {
    fn from(level: StrengthLevel) -> Self {
        match level {
            StrengthLevel::VeryWeak | StrengthLevel::Weak => Self::Weak,
            StrengthLevel::Fair => Self::Fair,
            StrengthLevel::Strong => Self::Strong,
            StrengthLevel::VeryStrong => Self::VeryStrong,
        }
    }
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
