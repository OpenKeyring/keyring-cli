//! OpenKeyring Core Library
//!
//! A privacy-first password manager with local-first architecture.

#![allow(dead_code)] // TODO: 移除此行，当 TUI 模块完全集成后
#![allow(clippy::result_large_err)] // TuiError 设计的已知限制
#![allow(clippy::needless_borrows_for_generic_args)] // format! 的借用检查
#![allow(clippy::redundant_closure)] // 框架代码的兼容性
#![allow(clippy::derivable_impls)] // 部分手动 impl 用于自定义逻辑
#![allow(clippy::needless_range_loop)] // 某些循环需要索引访问
#![allow(clippy::use_self)] // 框架代码的兼容性
#![allow(clippy::inherent_to_string)] // StateValue/StatePath 的自定义 to_string
#![allow(clippy::uninlined_format_args)] // 框架代码的可读性
#![allow(clippy::too_long_first_doc_paragraph)] // 文档风格
#![allow(clippy::let_underscore_untyped)] // 某些情况下使用 _
#![allow(clippy::items_after_statements)] // TUI 模块的代码组织
#![allow(clippy::type_complexity)] // 框架类型的复杂性

pub mod cli;
pub mod clipboard;
pub mod cloud;
pub mod config;
pub mod crypto;
pub mod db;
pub mod device;
pub mod error;
pub mod health;
pub mod mcp;
pub mod onboarding;
pub mod platform;
pub mod sync;
pub mod tui;
pub mod types;

// CLI diagnostics module
pub use cli::diagnostics;

pub use error::Result;
