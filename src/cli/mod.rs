//! CLI Application Module
//!
//! Provides the main CLI interface for OpenKeyring.

pub mod commands;
pub mod config;
pub mod utils;

pub use commands::{generate, list, show, update, delete, search, sync, health};
pub use config::ConfigManager;
pub use utils::PrettyPrinter;