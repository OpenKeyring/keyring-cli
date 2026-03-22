//! Keybinding manager
//!
//! Manages loading, storing, and querying keyboard shortcuts.

use super::binding::{Action, KeyBinding};
use crossterm::event::KeyEvent;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Keybinding manager
///
/// Loads configuration from YAML file and provides mapping from KeyEvent to Action.
pub struct KeyBindingManager {
    /// Mapping from KeyEvent to Action
    key_to_action: HashMap<KeyEvent, Action>,
    /// Reverse mapping from Action to KeyEvent (for help display)
    action_to_key: HashMap<Action, KeyEvent>,
    /// Configuration file path
    config_path: PathBuf,
}

impl KeyBindingManager {
    /// Create a new KeyBindingManager with default configuration
    pub fn new() -> Self {
        let config_path = Self::config_path();

        // Try to load from file, fall back to defaults
        let key_to_action = if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to load keybindings from {:?}: {}",
                    config_path, e
                );
                eprintln!("Using default keybindings");
                Self::default_keymap()
            })
        } else {
            // Create default config file
            if let Err(e) = Self::create_default_config(&config_path) {
                eprintln!("Warning: Failed to create default config: {}", e);
            }
            Self::default_keymap()
        };

        // Build reverse mapping
        let action_to_key = key_to_action.iter().map(|(k, v)| (*v, *k)).collect();

        Self {
            key_to_action,
            action_to_key,
            config_path,
        }
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("open-keyring").join("keybindings.yaml")
        } else {
            // Fallback to ~/.config/open-keyring
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".config")
                .join("open-keyring")
                .join("keybindings.yaml")
        }
    }

    /// Create the default keymap
    fn default_keymap() -> HashMap<KeyEvent, Action> {
        use crossterm::event::{KeyCode, KeyModifiers};

        let mut keymap = HashMap::new();

        // Core operations
        keymap.insert(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            Action::New,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
            Action::List,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
            Action::Search,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL),
            Action::Show,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
            Action::Update,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            Action::Delete,
        );

        // Navigation
        keymap.insert(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            Action::Quit,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::F(1), KeyModifiers::empty()),
            Action::Help,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL),
            Action::Clear,
        );

        // Password operations
        keymap.insert(
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL),
            Action::CopyPassword,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            Action::CopyUsername,
        );

        // Config
        keymap.insert(
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
            Action::Config,
        );

        // Sync-related actions
        keymap.insert(
            KeyEvent::new(KeyCode::F(2), KeyModifiers::empty()),
            Action::OpenSettings,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::F(5), KeyModifiers::empty()),
            Action::SyncNow,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()),
            Action::ShowHelp,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
            Action::RefreshView,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            Action::SaveConfig,
        );
        keymap.insert(
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
            Action::DisableSync,
        );

        keymap
    }

    /// Load keybindings from a YAML file
    fn load_from_file(path: &PathBuf) -> Result<HashMap<KeyEvent, Action>, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let binding: KeyBinding =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        // Convert HashMap<Action, KeyEvent> to HashMap<KeyEvent, Action>
        let action_to_key = binding.parse_shortcuts()?;
        let key_to_action: HashMap<KeyEvent, Action> = action_to_key
            .into_iter()
            .map(|(action, key)| (key, action))
            .collect();

        Ok(key_to_action)
    }

    /// Create the default configuration file
    fn create_default_config(path: &PathBuf) -> Result<(), String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        fs::write(path, super::DEFAULT_KEYBINDINGS)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Get the action for a given KeyEvent
    pub fn get_action(&self, event: &KeyEvent) -> Option<Action> {
        self.key_to_action.get(event).copied()
    }

    /// Get the KeyEvent for a given action
    pub fn get_key(&self, action: Action) -> Option<KeyEvent> {
        self.action_to_key.get(&action).copied()
    }

    /// Get all keybindings for display
    ///
    /// Returns bindings sorted by action description for consistent ordering.
    pub fn all_bindings(&self) -> Vec<(Action, KeyEvent)> {
        let mut bindings: Vec<(Action, KeyEvent)> =
            self.action_to_key.iter().map(|(a, k)| (*a, *k)).collect();

        // Sort by action description for deterministic ordering
        bindings.sort_by_key(|(action, _)| action.description());
        bindings
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> Result<(), String> {
        if self.config_path.exists() {
            let key_to_action = Self::load_from_file(&self.config_path)?;
            let action_to_key = key_to_action.iter().map(|(k, v)| (*v, *k)).collect();
            self.key_to_action = key_to_action;
            self.action_to_key = action_to_key;
            Ok(())
        } else {
            Err("Config file does not exist".to_string())
        }
    }

    /// Reset to default keybindings
    pub fn reset(&mut self) -> Result<(), String> {
        Self::create_default_config(&self.config_path)?;
        self.key_to_action = Self::default_keymap();
        self.action_to_key = self.key_to_action.iter().map(|(k, v)| (*v, *k)).collect();
        Ok(())
    }

    /// Format a KeyEvent as a string (for display)
    pub fn format_key(event: &KeyEvent) -> String {
        use crossterm::event::KeyCode;

        let mut parts = Vec::new();

        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            parts.push("Ctrl".to_string());
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::SHIFT)
        {
            parts.push("Shift".to_string());
        }
        if event
            .modifiers
            .contains(crossterm::event::KeyModifiers::ALT)
        {
            parts.push("Alt".to_string());
        }

        let key_str = match event.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            _ => format!("{:?}", event.code),
        };

        if parts.is_empty() {
            key_str
        } else {
            parts.push(key_str);
            parts.join("+")
        }
    }
}

impl Default for KeyBindingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_default_keybindings() {
        let manager = KeyBindingManager::new();

        // Test default bindings exist
        let ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_n), Some(Action::New));

        let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_l), Some(Action::List));

        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&ctrl_q), Some(Action::Quit));
    }

    #[test]
    fn test_get_key_for_action() {
        let manager = KeyBindingManager::new();

        let new_key = manager.get_key(Action::New);
        assert!(new_key.is_some());
        assert_eq!(new_key.unwrap().code, KeyCode::Char('n'));
    }

    #[test]
    fn test_format_key() {
        let ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
        assert_eq!(KeyBindingManager::format_key(&ctrl_n), "Ctrl+n");

        let ctrl_shift_n = KeyEvent::new(
            KeyCode::Char('N'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert_eq!(KeyBindingManager::format_key(&ctrl_shift_n), "Ctrl+Shift+N");

        let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
        assert_eq!(KeyBindingManager::format_key(&f5), "F5");
    }

    #[test]
    fn test_all_bindings() {
        let manager = KeyBindingManager::new();
        let bindings = manager.all_bindings();

        // Should have at least the core actions
        assert!(bindings.iter().any(|(a, _)| *a == Action::New));
        assert!(bindings.iter().any(|(a, _)| *a == Action::List));
        assert!(bindings.iter().any(|(a, _)| *a == Action::Quit));
    }

    #[test]
    fn test_unknown_key_returns_none() {
        let manager = KeyBindingManager::new();
        let unknown_key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(manager.get_action(&unknown_key), None);
    }
}
