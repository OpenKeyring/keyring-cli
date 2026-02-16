//! TUI 错误类型测试
//!
//! 测试 TuiError 及其相关类型的各种行为。

use crate::tui::error::{ErrorKind, ErrorSeverity, RecoveryStrategy, TuiError};
use crate::tui::traits::ComponentId;

// ============================================================================
// 错误严重级别测试
// ============================================================================

#[test]
fn test_error_severity_display() {
    assert_eq!(ErrorSeverity::Info.to_string(), "信息");
    assert_eq!(ErrorSeverity::Warning.to_string(), "警告");
    assert_eq!(ErrorSeverity::Error.to_string(), "错误");
    assert_eq!(ErrorSeverity::Fatal.to_string(), "致命");
}

// ============================================================================
// 恢复策略测试
// ============================================================================

#[test]
fn test_recovery_strategy_display() {
    assert_eq!(RecoveryStrategy::None.to_string(), "无需恢复");
    assert_eq!(RecoveryStrategy::Retry.to_string(), "重试");
    assert_eq!(RecoveryStrategy::Skip.to_string(), "跳过");
    assert_eq!(RecoveryStrategy::Undo.to_string(), "撤销");
    assert_eq!(RecoveryStrategy::Exit.to_string(), "退出");
    assert_eq!(RecoveryStrategy::ContactSupport.to_string(), "联系支持");
}

// ============================================================================
// TuiError 基础测试
// ============================================================================

#[test]
fn test_tui_error_new() {
    let error = TuiError::new(ErrorKind::DatabaseNotFound);
    assert_eq!(error.display_message(), "数据库不存在，请先运行初始化向导");
    assert!(error.is_fatal());
}

#[test]
fn test_tui_error_with_message() {
    let error = TuiError::new(ErrorKind::DatabaseNotFound)
        .with_message("自定义消息");
    assert_eq!(error.display_message(), "自定义消息");
}

#[test]
fn test_tui_error_with_details() {
    let error = TuiError::new(ErrorKind::IoError("file not found".to_string()))
        .with_details("详细错误信息");
    assert_eq!(error.details, Some("详细错误信息".to_string()));
}

#[test]
fn test_tui_error_with_source() {
    let error = TuiError::new(ErrorKind::ConfigInvalid("invalid json".to_string()))
        .with_source("config.json");
    assert_eq!(error.source, Some("config.json".to_string()));
}

// ============================================================================
// 便捷构造函数测试
// ============================================================================

#[test]
fn test_component_not_found() {
    let id = ComponentId::new(42);
    let error = TuiError::component_not_found(id);
    assert_eq!(error.display_message(), "组件未找到: ComponentId(42)");
    assert!(!error.is_fatal());
}

#[test]
fn test_invalid_input() {
    let error = TuiError::invalid_input("password", "too short");
    assert_eq!(error.display_message(), "无效输入 password: too short");
    assert!(!error.is_fatal());
}

#[test]
fn test_invalid_state() {
    let error = TuiError::invalid_state("component not mounted");
    assert_eq!(error.display_message(), "无效状态: component not mounted");
}

#[test]
fn test_database_not_found() {
    let error = TuiError::database_not_found();
    assert_eq!(error.display_message(), "数据库不存在，请先运行初始化向导");
    assert!(error.is_fatal());
}

// ============================================================================
// 错误严重级别推断测试
// ============================================================================

#[test]
fn test_fatal_error_database_corrupted() {
    let error = TuiError::new(ErrorKind::DatabaseCorrupted);
    assert!(error.is_fatal());
    assert_eq!(error.recovery, RecoveryStrategy::ContactSupport);
    assert_eq!(error.severity, ErrorSeverity::Fatal);
}

#[test]
fn test_fatal_error_terminal_too_small() {
    let error = TuiError::new(ErrorKind::TerminalTooSmall {
        required: (80, 24),
        actual: (40, 10),
    });
    assert!(error.is_fatal());
    assert_eq!(error.recovery, RecoveryStrategy::Exit);
}

#[test]
fn test_warning_error_record_not_found() {
    let error = TuiError::new(ErrorKind::RecordNotFound("test".to_string()));
    assert!(!error.is_fatal());
    assert_eq!(error.severity, ErrorSeverity::Warning);
    assert_eq!(error.recovery, RecoveryStrategy::Skip);
}

#[test]
fn test_error_io_error() {
    let error = TuiError::new(ErrorKind::IoError("permission denied".to_string()));
    assert!(!error.is_fatal());
    assert_eq!(error.severity, ErrorSeverity::Error);
    assert_eq!(error.recovery, RecoveryStrategy::Retry);
}

// ============================================================================
// 错误消息测试 (通过 TuiError 间接测试)
// ============================================================================

#[test]
fn test_component_not_found_message() {
    let id = ComponentId::new(1);
    let error = TuiError::new(ErrorKind::ComponentNotFound(id));
    assert_eq!(error.display_message(), "组件未找到: ComponentId(1)");
}

#[test]
fn test_component_init_failed_message() {
    let error = TuiError::new(ErrorKind::ComponentInitFailed("test".to_string()));
    assert_eq!(error.display_message(), "组件初始化失败: test");
}

#[test]
fn test_database_not_found_message() {
    let error = TuiError::new(ErrorKind::DatabaseNotFound);
    assert_eq!(error.display_message(), "数据库不存在，请先运行初始化向导");
}

#[test]
fn test_database_corrupted_message() {
    let error = TuiError::new(ErrorKind::DatabaseCorrupted);
    assert_eq!(error.display_message(), "数据库已损坏，请联系支持");
}

#[test]
fn test_clipboard_not_supported_message() {
    let error = TuiError::new(ErrorKind::ClipboardNotSupported);
    assert_eq!(error.display_message(), "当前系统不支持剪贴板操作");
}

#[test]
fn test_invalid_input_message() {
    let error = TuiError::new(ErrorKind::InvalidInput {
        field: "email".to_string(),
        reason: "invalid format".to_string(),
    });
    assert_eq!(error.display_message(), "无效输入 email: invalid format");
}

#[test]
fn test_terminal_too_small_message() {
    let error = TuiError::new(ErrorKind::TerminalTooSmall {
        required: (80, 24),
        actual: (40, 10),
    });
    assert_eq!(
        error.display_message(),
        "终端太小: 需要 80x24, 当前 40x10"
    );
}

// ============================================================================
// Display trait 测试
// ============================================================================

#[test]
fn test_tui_error_display() {
    let error = TuiError::new(ErrorKind::DatabaseNotFound);
    assert_eq!(
        format!("{}", error),
        "数据库不存在，请先运行初始化向导"
    );
}

#[test]
fn test_tui_error_display_with_custom_message() {
    let error = TuiError::new(ErrorKind::DatabaseNotFound)
        .with_message("无法找到数据库文件");
    assert_eq!(format!("{}", error), "无法找到数据库文件");
}

// ============================================================================
// From trait 测试
// ============================================================================

#[test]
fn test_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let tui_err: TuiError = io_err.into();
    assert!(tui_err.display_message().contains("IO 错误"));
    assert!(tui_err.details.is_some());
}

#[test]
fn test_from_json_error() {
    // 通过解析无效 JSON 生成错误
    let result: std::result::Result<serde_json::Value, _> = serde_json::from_str("{invalid}");
    let json_err = result.unwrap_err();
    let tui_err: TuiError = json_err.into();
    assert!(tui_err.display_message().contains("配置无效"));
    assert_eq!(tui_err.source, Some("serde_json".to_string()));
}
