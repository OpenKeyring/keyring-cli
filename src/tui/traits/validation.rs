//! 验证 trait 定义
//!
//! 占位符模块，完整实现将在 Task B.3 中完成。

/// 表单验证器 trait
pub trait FormValidator: Send + Sync {
    /// 验证输入
    fn validate(&self, input: &str) -> ValidationResult;
}

/// 验证规则
pub trait ValidationRule: Send + Sync {
    /// 检查规则
    fn check(&self, input: &str) -> bool;
}

/// 验证结果
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationResult {
    pub _is_valid: bool,
    pub _error_message: Option<String>,
}
