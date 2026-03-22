//! 表单验证 trait 定义
//!
//! 定义 TUI 表单验证相关的接口。

use std::collections::HashMap;

// ============================================================================
// 验证结果
// ============================================================================

/// 验证结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// 是否有效
    pub is_valid: bool,
    /// 错误消息列表
    pub errors: Vec<String>,
    /// 警告消息列表（不阻止提交但提示用户）
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// 创建有效的验证结果
    #[must_use]
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 创建无效的验证结果
    #[must_use]
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// 添加单个错误
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.is_valid = false;
        self
    }

    /// 添加警告
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// 检查是否有错误
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 检查是否有警告
    #[must_use]
    pub const fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 获取第一个错误消息
    #[must_use]
    pub fn first_error(&self) -> Option<&str> {
        self.errors.first().map(|s| s.as_str())
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::valid()
    }
}

// ============================================================================
// 验证时机
// ============================================================================

/// 验证时机
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationTrigger {
    /// 即时验证（每次输入）
    Immediate,
    /// 失焦时验证
    OnBlur,
    /// 提交时验证
    OnSubmit,
}

impl Default for ValidationTrigger {
    fn default() -> Self {
        Self::OnBlur
    }
}

// ============================================================================
// 验证器 Trait
// ============================================================================

/// 单个验证器 trait
pub trait Validator: Send + Sync {
    /// 执行验证
    fn validate(&self, value: &str) -> ValidationResult;

    /// 获取验证器名称
    fn name(&self) -> &str {
        "validator"
    }
}

/// 内置验证器
#[derive(Debug, Clone)]
pub enum BuiltinValidator {
    /// 必填字段
    Required,
    /// 最小长度
    MinLength(usize),
    /// 最大长度
    MaxLength(usize),
    /// 邮箱格式
    Email,
    /// URL 格式
    Url,
    /// 正则匹配
    Regex { pattern: String, message: String },
    /// 自定义验证器
    Custom {
        name: String,
        validator_fn: fn(&str) -> bool,
        error_message: String,
    },
}

impl Validator for BuiltinValidator {
    fn validate(&self, value: &str) -> ValidationResult {
        match self {
            Self::Required => {
                if value.trim().is_empty() {
                    ValidationResult::invalid(vec!["此字段为必填项".into()])
                } else {
                    ValidationResult::valid()
                }
            }
            Self::MinLength(min) => {
                if value.len() < *min {
                    ValidationResult::invalid(vec![format!("最少需要 {} 个字符", min)])
                } else {
                    ValidationResult::valid()
                }
            }
            Self::MaxLength(max) => {
                if value.len() > *max {
                    ValidationResult::invalid(vec![format!("最多允许 {} 个字符", max)])
                } else {
                    ValidationResult::valid()
                }
            }
            Self::Email => {
                if value.contains('@') && value.contains('.') {
                    ValidationResult::valid()
                } else {
                    ValidationResult::invalid(vec!["请输入有效的邮箱地址".into()])
                }
            }
            Self::Url => {
                if value.starts_with("http://") || value.starts_with("https://") {
                    ValidationResult::valid()
                } else {
                    ValidationResult::invalid(vec![
                        "请输入有效的 URL（以 http:// 或 https:// 开头）".into(),
                    ])
                }
            }
            Self::Regex {
                pattern: _,
                message,
            } => {
                // 简化实现，实际应使用 regex crate
                ValidationResult::invalid(vec![message.clone()])
            }
            Self::Custom {
                name: _,
                validator_fn,
                error_message,
            } => {
                if validator_fn(value) {
                    ValidationResult::valid()
                } else {
                    ValidationResult::invalid(vec![error_message.clone()])
                }
            }
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Required => "required",
            Self::MinLength(_) => "min_length",
            Self::MaxLength(_) => "max_length",
            Self::Email => "email",
            Self::Url => "url",
            Self::Regex { .. } => "regex",
            Self::Custom { name, .. } => name,
        }
    }
}

// ============================================================================
// 字段验证配置
// ============================================================================

/// 字段验证配置
#[derive(Debug, Clone)]
pub struct FieldValidation {
    /// 验证器列表
    pub validators: Vec<BuiltinValidator>,
    /// 验证时机（默认失焦）
    pub trigger: ValidationTrigger,
}

impl FieldValidation {
    /// 创建新的字段验证配置
    #[must_use]
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            trigger: ValidationTrigger::default(),
        }
    }

    /// 添加验证器
    #[must_use]
    pub fn with_validator(mut self, validator: BuiltinValidator) -> Self {
        self.validators.push(validator);
        self
    }

    /// 设置验证时机
    #[must_use]
    pub const fn with_trigger(mut self, trigger: ValidationTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    /// 执行所有验证器
    pub fn validate(&self, value: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();
        for validator in &self.validators {
            let r = validator.validate(value);
            if !r.is_valid {
                // 收集所有错误
                result.is_valid = false;
                result.errors.extend(r.errors);
            }
            result.warnings.extend(r.warnings);
        }
        result
    }
}

impl Default for FieldValidation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 表单验证管理器
// ============================================================================

/// 表单验证管理器 trait
pub trait FormValidator: Send + Sync {
    /// 注册字段验证规则
    fn register(&mut self, field: String, validation: FieldValidation);

    /// 验证单个字段
    fn validate_field(&self, field: &str, value: &str) -> ValidationResult;

    /// 验证所有字段
    fn validate_all(&self, values: &HashMap<String, String>) -> ValidationResult;

    /// 检查表单是否可提交
    fn is_valid(&self) -> bool;

    /// 获取字段错误
    fn get_errors(&self, field: &str) -> Vec<String>;

    /// 清除所有验证状态
    fn clear(&mut self);
}
