//! Keyboard Shortcuts System for TUI
//!
//! This module provides a configurable keyboard shortcuts system for the TUI.
//! Shortcuts can be configured via YAML file at:
//! - macOS/Linux: ~/.config/open-keyring/keybindings.yaml
//! - Windows: %APPDATA%\open-keyring\keybindings.yaml

mod binding;
mod manager;
mod parser;

pub use binding::{Action, KeyBinding};
pub use manager::KeyBindingManager;
pub use parser::parseShortcut;

/// Default keybindings configuration
pub const DEFAULT_KEYBINDINGS: &str = r#"version: "1.0"

shortcuts:
  # Core operations
  new: "Ctrl+N"
  list: "Ctrl+L"
  search: "Ctrl+S"
  show: "Ctrl+O"
  update: "Ctrl+E"
  delete: "Ctrl+D"

  # Navigation
  quit: "Ctrl+Q"
  help: "Ctrl+H"
  clear: "Ctrl+R"

  # Password operations
  copy_password: "Ctrl+Y"
  copy_username: "Ctrl+U"

  # Config
  config: "Ctrl+P"
"#;
