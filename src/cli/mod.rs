//! CLI Application Module
//!
//! Provides the main CLI interface for OpenKeyring.

pub mod commands;
pub mod config;
pub mod diagnostics;
pub mod mcp;
pub mod onboarding;
pub mod utils;

pub use commands::{delete, generate, health, list, search, show, sync, update};
pub use config::ConfigManager;
pub use mcp::{handle_mcp_command, MCPCommands};
pub use utils::PrettyPrinter;
