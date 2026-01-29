//! Terminal User Interface (TUI) for OpenKeyring
//!
//! This module provides an interactive TUI mode that displays sensitive information
//! in alternate screen mode to prevent terminal scrollback leakage.

mod app;
pub mod commands;
pub mod handler;
pub mod keybindings;
pub mod screens;
mod utils;
mod widgets;

pub use app::{run_tui, TuiApp, TuiError};
pub use handler::{AppAction, TuiEventHandler};

/// TUI result type
pub type TuiResult<T> = std::result::Result<T, TuiError>;
