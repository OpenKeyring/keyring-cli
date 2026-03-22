//! MCP Tool Handlers
//!
//! This module provides handlers for various MCP tools. Handlers connect
//! tool definitions to executors with proper authorization flow.

pub mod ssh;

pub use ssh::{handle_ssh_exec, HandlerError};
