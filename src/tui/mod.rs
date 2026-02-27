//! Terminal User Interface (TUI) for OpenKeyring
//!
//! This module provides an interactive TUI mode that displays sensitive information
//! in alternate screen mode to prevent terminal scrollback leakage.
//!
//! ## 模块结构
//!
//! - [`error`] - Phase 1.2 错误类型定义
//! - [`traits`] - Trait 层定义，包含所有核心接口
//! - [`core`] - 核心实现层，提供默认实现
//! - [`commands`] - CLI 命令处理
//! - [`handler`] - 事件处理器
//! - [`keybindings`] - 键盘快捷键管理
//! - [`screens`] - 各种屏幕界面

//! - [`widgets`] - UI 组件

// ============ Phase 1.2 新模块 ============
pub mod error;
pub mod traits;
pub mod core;

// ============ TUI MVP State Module ============
pub mod state;

// ============ Phase 1.3 数据层模块 ============
pub mod models;
pub mod services;

// ============ Phase 1.4 组件模块 ============
pub mod components;

// ============ 现有模块 ============
mod app;
pub mod commands;
pub mod handler;
pub mod keybindings;
pub mod screens;
pub mod tags;
mod utils;
mod widgets;
pub mod wizard_flow;

#[cfg(test)]
pub mod testing;

#[cfg(test)]
mod tests;

// ============ 公共导出 ============
pub use app::{run_tui, Screen, TuiApp, TuiError as LegacyTuiError};
pub use handler::{AppAction, TuiEventHandler};

// Phase 1.2 错误类型导出
pub use error::{ErrorSeverity, ErrorKind, RecoveryStrategy, TuiError, TuiResult};
