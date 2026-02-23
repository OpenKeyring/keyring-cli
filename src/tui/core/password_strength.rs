//! 默认密码强度计算器实现
//!
//! 根据参考文档实现完整的密码强度计算逻辑。

use crate::tui::traits::{
    PasswordStrength, PasswordStrengthCalculator, StrengthLevel,
};

// ============================================================================
// 密码强度详情（扩展 Trait 层定义）
// ============================================================================

/// 密码强度详情
#[derive(Debug, Clone)]
pub struct PasswordStrengthDetails {
    /// 强度分数 (0-100)
    pub score: u8,
    /// 强度等级
    pub level: StrengthLevel,
    /// 改进建议
    pub suggestions: Vec<String>,
    /// 检测到的问题
    pub warnings: Vec<String>,
    /// 预估破解时间（用于显示）
    pub crack_time: Option<String>,
}

impl PasswordStrengthDetails {
    /// 创建空的强度评估
    #[must_use]
    pub fn empty() -> Self {
        Self {
            score: 0,
            level: StrengthLevel::VeryWeak,
            suggestions: Vec::new(),
            warnings: Vec::new(),
            crack_time: None,
        }
    }

    /// 从分数创建
    #[must_use]
    pub fn from_score(score: u8) -> Self {
        let level = StrengthLevel::from_score(score);
        Self {
            score,
            level,
            suggestions: Vec::new(),
            warnings: Vec::new(),
            crack_time: estimate_crack_time(score),
        }
    }

    /// 是否为强密码
    #[must_use]
    pub fn is_strong(&self) -> bool {
        matches!(self.level, StrengthLevel::Strong | StrengthLevel::VeryStrong)
    }

    /// 添加建议
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// 添加警告
    #[must_use]
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// 估算破解时间
#[must_use]
pub fn estimate_crack_time(score: u8) -> Option<String> {
    match score {
        0..=20 => Some("瞬间".to_string()),
        21..=40 => Some("几分钟".to_string()),
        41..=60 => Some("几小时".to_string()),
        61..=80 => Some("几天".to_string()),
        81..=95 => Some("几个月".to_string()),
        96..=100 => Some("几年".to_string()),
        _ => None,
    }
}

// ============================================================================
// 默认密码强度计算器
// ============================================================================

/// 默认密码强度计算器（简单实现）
///
/// 生产环境应替换为调用 CryptoService
#[derive(Debug, Clone, Copy)]
pub struct DefaultPasswordStrengthCalculator {
    /// 最小长度要求
    pub min_length: usize,
    /// 是否要求包含小写字母
    pub require_lower: bool,
    /// 是否要求包含大写字母
    pub require_upper: bool,
    /// 是否要求包含数字
    pub require_digit: bool,
    /// 是否要求包含特殊字符
    pub require_special: bool,
}

impl Default for DefaultPasswordStrengthCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultPasswordStrengthCalculator {
    /// 创建新的计算器
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_length: 8,
            require_lower: true,
            require_upper: true,
            require_digit: true,
            require_special: true,
        }
    }

    /// 设置最小长度
    #[must_use]
    pub const fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    /// 设置是否要求小写字母
    #[must_use]
    pub const fn with_require_lower(mut self, require: bool) -> Self {
        self.require_lower = require;
        self
    }

    /// 设置是否要求大写字母
    #[must_use]
    pub const fn with_require_upper(mut self, require: bool) -> Self {
        self.require_upper = require;
        self
    }

    /// 设置是否要求数字
    #[must_use]
    pub const fn with_require_digit(mut self, require: bool) -> Self {
        self.require_digit = require;
        self
    }

    /// 设置是否要求特殊字符
    #[must_use]
    pub const fn with_require_special(mut self, require: bool) -> Self {
        self.require_special = require;
        self
    }

    /// 计算分数
    fn calculate_score(&self, password: &str) -> (u8, Vec<String>, Vec<String>) {
        let mut score: u8 = 0;
        let mut suggestions = Vec::new();
        let mut warnings = Vec::new();

        // 长度评分
        let len = password.len();
        if len >= self.min_length {
            score += 20;
        } else {
            warnings.push(format!("长度不足 {} 个字符", self.min_length));
            suggestions.push(format!("至少使用 {} 个字符", self.min_length));
        }
        if len >= 12 {
            score += 10;
        }
        if len >= 16 {
            score += 10;
        }

        // 字符多样性
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?/`~".contains(c));

        if has_lower {
            score += 10;
        } else if self.require_lower {
            suggestions.push("添加小写字母".to_string());
        }
        if has_upper {
            score += 10;
        } else if self.require_upper {
            suggestions.push("添加大写字母".to_string());
        }
        if has_digit {
            score += 10;
        } else if self.require_digit {
            suggestions.push("添加数字".to_string());
        }
        if has_special {
            score += 20;
        } else if self.require_special {
            suggestions.push("添加特殊字符 (!@#$%^&*)".to_string());
        }

        // 字符种类奖励
        let variety = [has_lower, has_upper, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        if variety >= 3 {
            score += 10;
        }
        if variety == 4 {
            score += 10;
        }

        (score, suggestions, warnings)
    }
}

impl PasswordStrengthCalculator for DefaultPasswordStrengthCalculator {
    fn calculate(&self, password: &str) -> PasswordStrength {
        let (score, _suggestions, _warnings) = self.calculate_score(password);
        let level = StrengthLevel::from_score(score);

        // 将分数映射到 PasswordStrength 枚举
        match level {
            StrengthLevel::VeryWeak => PasswordStrength::Weak,
            StrengthLevel::Weak => PasswordStrength::Weak,
            StrengthLevel::Fair => PasswordStrength::Fair,
            StrengthLevel::Strong => PasswordStrength::Strong,
            StrengthLevel::VeryStrong => PasswordStrength::VeryStrong,
        }
    }
}
