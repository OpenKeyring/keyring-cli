//! 默认表单验证器实现
//!
//! 占位符模块，完整实现将在 Task C.6 中完成。

use crate::tui::traits::{FormValidator, ValidationResult, Validator};
use std::collections::HashMap;

/// 默认表单验证器
#[derive(Default)]
pub struct DefaultFormValidator {
    /// 字段验证规则映射
    field_validations: HashMap<String, Vec<Box<dyn Validator + Send + Sync>>>,
}

impl std::fmt::Debug for DefaultFormValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultFormValidator")
            .field("fields", &self.field_validations.len())
            .finish()
    }
}

impl DefaultFormValidator {
    /// 创建新的验证器
    #[must_use]
    pub fn new() -> Self {
        Self {
            field_validations: HashMap::new(),
        }
    }

    /// 添加字段验证规则
    pub fn add_rule(&mut self, field: String, rule: Box<dyn Validator + Send + Sync>) {
        self.field_validations.entry(field).or_default().push(rule);
    }

    /// 验证单个字段
    pub fn validate_field(&self, field: &str, value: &str) -> ValidationResult {
        if let Some(rules) = self.field_validations.get(field) {
            for rule in rules {
                let result = rule.validate(value);
                if !result.is_valid {
                    return result;
                }
            }
        }
        ValidationResult::valid()
    }
}

impl FormValidator for DefaultFormValidator {
    fn register(&mut self, field: String, validation: crate::tui::traits::FieldValidation) {
        for validator in validation.validators {
            self.add_rule(field.clone(), Box::new(validator));
        }
    }

    fn validate_field(&self, field: &str, value: &str) -> ValidationResult {
        if let Some(rules) = self.field_validations.get(field) {
            for rule in rules {
                let result = rule.validate(value);
                if !result.is_valid {
                    return result;
                }
            }
        }
        ValidationResult::valid()
    }

    fn validate_all(&self, values: &HashMap<String, String>) -> ValidationResult {
        let mut result = ValidationResult::valid();
        for (field, value) in values {
            let r = self.validate_field(field, value);
            if !r.is_valid {
                result.is_valid = false;
                result.errors.extend(r.errors);
            }
            result.warnings.extend(r.warnings);
        }
        result
    }

    fn is_valid(&self) -> bool {
        self.field_validations.is_empty()
    }

    fn get_errors(&self, field: &str) -> Vec<String> {
        if let Some(rules) = self.field_validations.get(field) {
            let mut errors = Vec::new();
            for rule in rules {
                let r = rule.validate("");
                errors.extend(r.errors);
            }
            errors
        } else {
            Vec::new()
        }
    }

    fn clear(&mut self) {
        self.field_validations.clear();
    }
}

/// 简单验证规则
#[derive(Debug, Clone)]
pub struct SimpleValidationRule {
    min_length: Option<usize>,
    max_length: Option<usize>,
    error_message: String,
}

impl SimpleValidationRule {
    /// 创建新的验证规则
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_length: None,
            max_length: None,
            error_message: "验证失败".to_string(),
        }
    }

    /// 设置最小长度
    #[must_use]
    pub const fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// 设置最大长度
    #[must_use]
    pub const fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// 设置错误消息
    #[must_use]
    pub fn with_error_message(mut self, msg: String) -> Self {
        self.error_message = msg;
        self
    }
}

impl Default for SimpleValidationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for SimpleValidationRule {
    fn validate(&self, input: &str) -> ValidationResult {
        if let Some(min) = self.min_length {
            if input.len() < min {
                return ValidationResult::invalid(vec![self.error_message.clone()]);
            }
        }
        if let Some(max) = self.max_length {
            if input.len() > max {
                return ValidationResult::invalid(vec![self.error_message.clone()]);
            }
        }
        ValidationResult::valid()
    }

    fn name(&self) -> &str {
        "simple_validation"
    }
}
