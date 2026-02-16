//! 默认表单验证器实现
//!
//! 占位符模块，完整实现将在 Task C.6 中完成。

use crate::tui::traits::{FormValidator, ValidationRule, ValidationResult};

/// 默认表单验证器
#[derive(Default)]
pub struct DefaultFormValidator {
    _rules: Vec<Box<dyn ValidationRule>>,
}

impl std::fmt::Debug for DefaultFormValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultFormValidator")
            .field("rules", &self._rules.len())
            .finish()
    }
}

impl DefaultFormValidator {
    /// 创建新的验证器
    #[must_use]
    pub fn new() -> Self {
        Self {
            _rules: Vec::new(),
        }
    }

    /// 添加验证规则
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self._rules.push(rule);
    }
}

impl FormValidator for DefaultFormValidator {
    fn validate(&self, input: &str) -> ValidationResult {
        for rule in &self._rules {
            if !rule.check(input) {
                return ValidationResult {
                    _is_valid: false,
                    _error_message: Some("验证失败".to_string()),
                };
            }
        }
        ValidationResult {
            _is_valid: true,
            _error_message: None,
        }
    }
}

/// 简单验证规则
#[derive(Debug, Clone, Default)]
pub struct SimpleValidationRule {
    _min_length: Option<usize>,
    _max_length: Option<usize>,
}

impl SimpleValidationRule {
    /// 创建新的验证规则
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _min_length: None,
            _max_length: None,
        }
    }
}

impl ValidationRule for SimpleValidationRule {
    fn check(&self, input: &str) -> bool {
        if let Some(min) = self._min_length {
            if input.len() < min {
                return false;
            }
        }
        if let Some(max) = self._max_length {
            if input.len() > max {
                return false;
            }
        }
        true
    }
}
