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
pub use parser::parse_shortcut;

/// Default keybindings configuration
pub const DEFAULT_KEYBINDINGS: &str = r#"version: "1.0"

shortcuts:
  # Core operations
  new: "Ctrl+N"
  list: "Ctrl+L"
  search: "Ctrl+F"
  show: "Ctrl+O"
  update: "Ctrl+E"
  delete: "Ctrl+X"

  # Navigation
  quit: "Ctrl+Q"
  help: "F1"
  clear: "Ctrl+K"

  # Password operations
  copy_password: "Ctrl+Y"
  copy_username: "Ctrl+U"

  # Config
  config: "Ctrl+P"

  # Sync-related actions
  open_settings: "F2"
  sync_now: "F5"
  show_help: "?"
  refresh_view: "Ctrl+R"
  save_config: "Ctrl+S"
  disable_sync: "Ctrl+D"
"#;
