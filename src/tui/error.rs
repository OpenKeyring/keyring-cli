//! TUI 错误处理模块
//!
//! 定义 TUI 框架使用的错误类型，包括错误分类、严重级别、恢复策略等。

use crate::tui::traits::ComponentId;

// ============================================================================
// 公共类型
// ============================================================================

/// TUI 结果类型
pub type TuiResult<T> = std::result::Result<T, TuiError>;

// ============================================================================
// 错误严重级别
// ============================================================================

/// 错误严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 信息级别（不需要用户操作）
    Info,
    /// 警告级别（建议用户注意）
    Warning,
    /// 错误级别（需要用户处理）
    Error,
    /// 致命错误（无法恢复）
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "信息"),
            Self::Warning => write!(f, "警告"),
            Self::Error => write!(f, "错误"),
            Self::Fatal => write!(f, "致命"),
        }
    }
}

// ============================================================================
// 错误恢复策略
// ============================================================================

/// 错误恢复策略
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// 无需恢复
    None,
    /// 重试操作
    Retry,
    /// 跳过当前操作
    Skip,
    /// 返回上一状态
    Undo,
    /// 退出应用
    Exit,
    /// 联系支持
    ContactSupport,
}

impl std::fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "无需恢复"),
            Self::Retry => write!(f, "重试"),
            Self::Skip => write!(f, "跳过"),
            Self::Undo => write!(f, "撤销"),
            Self::Exit => write!(f, "退出"),
            Self::ContactSupport => write!(f, "联系支持"),
        }
    }
}

// ============================================================================
// 错误类型
// ============================================================================

/// 错误类型枚举
#[derive(Debug, Clone)]
pub enum ErrorKind {
    // === 组件错误 ===
    /// 组件未找到
    ComponentNotFound(ComponentId),
    /// 组件初始化失败
    ComponentInitFailed(String),
    /// 组件渲染失败
    ComponentRenderFailed(String),

    // === 配置错误 ===
    /// 配置文件未找到
    ConfigNotFound(String),
    /// 配置无效
    ConfigInvalid(String),
    /// 配置保存失败
    ConfigSaveFailed(String),

    // === 数据库错误 ===
    /// 数据库不存在
    DatabaseNotFound,
    /// 数据库已损坏
    DatabaseCorrupted,
    /// 数据库被锁定
    DatabaseLocked,
    /// 记录未找到
    RecordNotFound(String),
    /// 记录重复
    RecordDuplicate(String),

    // === 加密错误 ===
    /// 加密失败
    EncryptionFailed,
    /// 解密失败
    DecryptionFailed,
    /// 无效密钥
    InvalidKey,
    /// 密钥派生失败
    KeyDerivationFailed,

    // === 剪贴板错误 ===
    /// 系统不支持剪贴板
    ClipboardNotSupported,
    /// 剪贴板超时
    ClipboardTimeout,
    /// 剪贴板复制失败
    ClipboardCopyFailed,

    // === 输入验证错误 ===
    /// 无效输入
    InvalidInput { field: String, reason: String },
    /// 密码太弱
    PasswordTooWeak { score: u8 },
    /// 必填字段为空
    RequiredFieldEmpty(String),

    // === 系统错误 ===
    /// IO 错误
    IoError(String),
    /// 终端太小
    TerminalTooSmall { required: (u16, u16), actual: (u16, u16) },
    /// 不支持的平台
    UnsupportedPlatform,

    // === 状态错误 ===
    /// 无效状态
    InvalidState(String),

    // === 其他 ===
    /// 其他错误
    Other(String),
}

impl ErrorKind {
    /// 获取默认用户消息
    fn default_message(&self) -> String {
        match self {
            Self::ComponentNotFound(id) => format!("组件未找到: {}", id),
            Self::ComponentInitFailed(name) => format!("组件初始化失败: {}", name),
            Self::ComponentRenderFailed(name) => format!("组件渲染失败: {}", name),

            Self::ConfigNotFound(path) => format!("配置文件未找到: {}", path),
            Self::ConfigInvalid(reason) => format!("配置无效: {}", reason),
            Self::ConfigSaveFailed(path) => format!("配置保存失败: {}", path),

            Self::DatabaseNotFound => "数据库不存在，请先运行初始化向导".to_string(),
            Self::DatabaseCorrupted => "数据库已损坏，请联系支持".to_string(),
            Self::DatabaseLocked => "数据库已被锁定，请检查是否有其他实例运行".to_string(),
            Self::RecordNotFound(name) => format!("记录未找到: {}", name),
            Self::RecordDuplicate(name) => format!("记录已存在: {}", name),

            Self::EncryptionFailed => "加密失败".to_string(),
            Self::DecryptionFailed => "解密失败".to_string(),
            Self::InvalidKey => "密钥无效".to_string(),
            Self::KeyDerivationFailed => "密钥派生失败".to_string(),

            Self::ClipboardNotSupported => "当前系统不支持剪贴板操作".to_string(),
            Self::ClipboardTimeout => "剪贴板操作超时".to_string(),
            Self::ClipboardCopyFailed => "复制到剪贴板失败".to_string(),

            Self::InvalidInput { field, reason } => format!("无效输入 {}: {}", field, reason),
            Self::PasswordTooWeak { score } => format!("密码太弱 (强度: {}/100)", score),
            Self::RequiredFieldEmpty(field) => format!("必填字段为空: {}", field),

            Self::IoError(msg) => format!("IO 错误: {}", msg),
            Self::TerminalTooSmall { required, actual } => {
                format!("终端太小: 需要 {}x{}, 当前 {}x{}", required.0, required.1, actual.0, actual.1)
            }
            Self::UnsupportedPlatform => "当前平台不支持".to_string(),

            Self::InvalidState(msg) => format!("无效状态: {}", msg),

            Self::Other(msg) => format!("未知错误: {}", msg),
        }
    }
}

// ============================================================================
// 完整错误结构
// ============================================================================

/// TUI 错误
///
/// 包含错误类型、严重级别、恢复策略等信息。
#[derive(Debug, Clone)]
pub struct TuiError {
    /// 错误类型
    pub kind: ErrorKind,
    /// 严重级别
    pub severity: ErrorSeverity,
    /// 用户友好的消息
    pub message: String,
    /// 技术详情（用于调试）
    pub details: Option<String>,
    /// 恢复策略
    pub recovery: RecoveryStrategy,
    /// 错误来源
    pub source: Option<String>,
}

impl TuiError {
    /// 创建新错误
    pub fn new(kind: ErrorKind) -> Self {
        let (severity, recovery) = Self::infer_severity_and_recovery(&kind);
        Self {
            kind,
            severity,
            message: String::new(),
            details: None,
            recovery,
            source: None,
        }
    }

    /// 设置用户消息
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// 设置技术详情
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// 设置错误来源
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 推断严重级别和恢复策略
    fn infer_severity_and_recovery(kind: &ErrorKind) -> (ErrorSeverity, RecoveryStrategy) {
        match kind {
            // 致命错误：需要联系支持
            ErrorKind::DatabaseCorrupted
            | ErrorKind::DatabaseLocked
            | ErrorKind::UnsupportedPlatform
            | ErrorKind::EncryptionFailed
            | ErrorKind::DecryptionFailed
            | ErrorKind::KeyDerivationFailed => {
                (ErrorSeverity::Fatal, RecoveryStrategy::ContactSupport)
            }

            // 致命错误：需要退出
            ErrorKind::DatabaseNotFound | ErrorKind::TerminalTooSmall { .. } => {
                (ErrorSeverity::Fatal, RecoveryStrategy::Exit)
            }

            // 警告：可以跳过
            ErrorKind::RecordNotFound(_)
            | ErrorKind::ClipboardNotSupported
            | ErrorKind::ClipboardTimeout => (ErrorSeverity::Warning, RecoveryStrategy::Skip),

            // 警告：无需特殊恢复
            ErrorKind::InvalidInput { .. }
            | ErrorKind::PasswordTooWeak { .. }
            | ErrorKind::RequiredFieldEmpty(_) => (ErrorSeverity::Warning, RecoveryStrategy::None),

            // 错误：可以重试
            ErrorKind::ClipboardCopyFailed
            | ErrorKind::IoError(_)
            | ErrorKind::ConfigSaveFailed(_) => (ErrorSeverity::Error, RecoveryStrategy::Retry),

            // 其他：默认错误级别
            _ => (ErrorSeverity::Error, RecoveryStrategy::None),
        }
    }

    /// 是否为致命错误
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.severity == ErrorSeverity::Fatal
    }

    /// 获取用户显示消息
    #[must_use]
    pub fn display_message(&self) -> String {
        if !self.message.is_empty() {
            self.message.clone()
        } else {
            self.kind.default_message()
        }
    }

    // ========== 便捷构造函数 ==========

    /// 组件未找到
    #[must_use]
    pub fn component_not_found(id: ComponentId) -> Self {
        Self::new(ErrorKind::ComponentNotFound(id))
    }

    /// 无效输入
    #[must_use]
    pub fn invalid_input(field: &str, reason: &str) -> Self {
        Self::new(ErrorKind::InvalidInput {
            field: field.to_string(),
            reason: reason.to_string(),
        })
    }

    /// 无效状态
    #[must_use]
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidState(msg.into()))
    }

    /// 数据库未找到
    #[must_use]
    pub fn database_not_found() -> Self {
        Self::new(ErrorKind::DatabaseNotFound)
    }
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_message())
    }
}

impl std::error::Error for TuiError {}

// ============================================================================
// From 其他错误类型
// ============================================================================

impl From<std::io::Error> for TuiError {
    fn from(err: std::io::Error) -> Self {
        Self::new(ErrorKind::IoError(err.to_string()))
            .with_details(format!("{:?}", err))
    }
}

impl From<serde_json::Error> for TuiError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(ErrorKind::ConfigInvalid(err.to_string()))
            .with_source("serde_json".to_string())
    }
}
