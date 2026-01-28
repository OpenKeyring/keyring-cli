//! Keybinding data structures
//!
//! Defines the Action enum and KeyBinding configuration struct.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Actions that can be triggered by keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Create a new record
    New,
    /// List all records
    List,
    /// Search records
    Search,
    /// Show record details
    Show,
    /// Update a record
    Update,
    /// Delete a record
    Delete,
    /// Quit the TUI
    Quit,
    /// Show help
    Help,
    /// Clear screen/output
    Clear,
    /// Copy password to clipboard
    CopyPassword,
    /// Copy username to clipboard
    CopyUsername,
    /// Open configuration
    Config,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::New => write!(f, "New"),
            Action::List => write!(f, "List"),
            Action::Search => write!(f, "Search"),
            Action::Show => write!(f, "Show"),
            Action::Update => write!(f, "Update"),
            Action::Delete => write!(f, "Delete"),
            Action::Quit => write!(f, "Quit"),
            Action::Help => write!(f, "Help"),
            Action::Clear => write!(f, "Clear"),
            Action::CopyPassword => write!(f, "CopyPassword"),
            Action::CopyUsername => write!(f, "CopyUsername"),
            Action::Config => write!(f, "Config"),
        }
    }
}

impl Action {
    /// Get the command name associated with this action (for TUI slash commands)
    pub fn command_name(&self) -> &'static str {
        match self {
            Action::New => "/new",
            Action::List => "/list",
            Action::Search => "/search",
            Action::Show => "/show",
            Action::Update => "/update",
            Action::Delete => "/delete",
            Action::Quit => "/exit",
            Action::Help => "/help",
            Action::Clear => "/clear",
            Action::CopyPassword => "/copy_password",
            Action::CopyUsername => "/copy_username",
            Action::Config => "/config",
        }
    }

    /// Get a user-friendly description for this action
    pub fn description(&self) -> &'static str {
        match self {
            Action::New => "Create a new record",
            Action::List => "List all records",
            Action::Search => "Search records",
            Action::Show => "Show record details",
            Action::Update => "Update a record",
            Action::Delete => "Delete a record",
            Action::Quit => "Quit TUI",
            Action::Help => "Show help",
            Action::Clear => "Clear screen",
            Action::CopyPassword => "Copy password to clipboard",
            Action::CopyUsername => "Copy username to clipboard",
            Action::Config => "Open configuration",
        }
    }
}

/// Keybinding configuration loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Configuration version
    pub version: String,
    /// Shortcut mappings
    pub shortcuts: HashMap<String, String>,
}

impl KeyBinding {
    /// Create a new default keybinding configuration
    pub fn new() -> Self {
        let mut shortcuts = HashMap::new();

        // Core operations
        shortcuts.insert("new".to_string(), "Ctrl+N".to_string());
        shortcuts.insert("list".to_string(), "Ctrl+L".to_string());
        shortcuts.insert("search".to_string(), "Ctrl+S".to_string());
        shortcuts.insert("show".to_string(), "Ctrl+O".to_string());
        shortcuts.insert("update".to_string(), "Ctrl+E".to_string());
        shortcuts.insert("delete".to_string(), "Ctrl+D".to_string());

        // Navigation
        shortcuts.insert("quit".to_string(), "Ctrl+Q".to_string());
        shortcuts.insert("help".to_string(), "Ctrl+H".to_string());
        shortcuts.insert("clear".to_string(), "Ctrl+R".to_string());

        // Password operations
        shortcuts.insert("copy_password".to_string(), "Ctrl+Y".to_string());
        shortcuts.insert("copy_username".to_string(), "Ctrl+U".to_string());

        // Config
        shortcuts.insert("config".to_string(), "Ctrl+P".to_string());

        Self {
            version: "1.0".to_string(),
            shortcuts,
        }
    }

    /// Parse the shortcuts into a map of actions to key events
    pub fn parse_shortcuts(&self) -> Result<HashMap<Action, KeyEvent>, String> {
        let mut result = HashMap::new();

        for (action_name, shortcut_str) in &self.shortcuts {
            let action = match action_name.as_str() {
                "new" => Action::New,
                "list" => Action::List,
                "search" => Action::Search,
                "show" => Action::Show,
                "update" => Action::Update,
                "delete" => Action::Delete,
                "quit" => Action::Quit,
                "help" => Action::Help,
                "clear" => Action::Clear,
                "copy_password" => Action::CopyPassword,
                "copy_username" => Action::CopyUsername,
                "config" => Action::Config,
                _ => continue, // Unknown action, skip
            };

            match super::parser::parseShortcut(shortcut_str) {
                Ok(key_event) => {
                    result.insert(action, key_event);
                }
                Err(e) => {
                    // Log warning but continue
                    eprintln!("Warning: Failed to parse shortcut '{}': {}", shortcut_str, e);
                }
            }
        }

        Ok(result)
    }
}

impl Default for KeyBinding {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", Action::New), "New");
        assert_eq!(format!("{}", Action::List), "List");
        assert_eq!(format!("{}", Action::Quit), "Quit");
    }

    #[test]
    fn test_action_command_name() {
        assert_eq!(Action::New.command_name(), "/new");
        assert_eq!(Action::List.command_name(), "/list");
        assert_eq!(Action::Quit.command_name(), "/exit");
    }

    #[test]
    fn test_action_description() {
        assert_eq!(Action::New.description(), "Create a new record");
        assert_eq!(Action::Quit.description(), "Quit TUI");
    }

    #[test]
    fn test_keybinding_default() {
        let binding = KeyBinding::new();
        assert_eq!(binding.version, "1.0");
        assert_eq!(binding.shortcuts.get("new"), Some(&"Ctrl+N".to_string()));
        assert_eq!(binding.shortcuts.get("quit"), Some(&"Ctrl+Q".to_string()));
    }
}
