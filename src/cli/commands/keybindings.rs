//! CLI Keybindings Commands
//!
//! Manage keyboard shortcuts configuration from the CLI.

use crate::error::{KeyringError, Result};
use crate::tui::keybindings::KeyBindingManager;
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
pub struct KeybindingsArgs {
    /// List all keyboard shortcuts
    #[clap(long, short)]
    pub list: bool,

    /// Validate keybindings configuration
    #[clap(long, short)]
    pub validate: bool,

    /// Reset keybindings to defaults
    #[clap(long, short)]
    pub reset: bool,

    /// Edit keybindings configuration
    #[clap(long, short)]
    pub edit: bool,
}

/// Manage keybindings configuration
pub async fn manage_keybindings(args: KeybindingsArgs) -> Result<()> {
    let config_path = get_config_path();

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                KeyringError::IoError(format!("Failed to create config directory: {}", e))
            })?;
        }
    }

    // Handle subcommands
    if args.list {
        list_keybindings(&config_path)?;
    } else if args.validate {
        validate_keybindings(&config_path)?;
    } else if args.reset {
        reset_keybindings(&config_path)?;
    } else if args.edit {
        edit_keybindings(&config_path)?;
    } else {
        // Default: list all bindings
        list_keybindings(&config_path)?;
    }

    Ok(())
}

/// Get the keybindings configuration file path
fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("open-keyring").join("keybindings.yaml")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("open-keyring")
            .join("keybindings.yaml")
    }
}

/// List all keyboard shortcuts
fn list_keybindings(config_path: &Path) -> Result<()> {
    let manager = KeyBindingManager::new();
    let bindings = manager.all_bindings();

    println!("🎹 Keyboard Shortcuts:");
    println!("   Configuration: {}", config_path.display());
    println!();

    // Sort by action name for consistent display
    let mut sorted_bindings: Vec<_> = bindings.iter().collect();
    sorted_bindings.sort_by_key(|(a, _)| format!("{:?}", a));

    for (action, key_event) in sorted_bindings {
        let key_str = KeyBindingManager::format_key(key_event);
        println!("   {:20} - {}", key_str, action.description());
    }

    println!();
    println!("To customize, edit: {}", config_path.display());
    println!("Or run: ok keybindings edit");

    Ok(())
}

/// Validate keybindings configuration
fn validate_keybindings(config_path: &Path) -> Result<()> {
    println!("🔍 Validating keybindings configuration...");
    println!("   File: {}", config_path.display());
    println!();

    if !config_path.exists() {
        println!("✅ Configuration file does not exist (will use defaults)");
        return Ok(());
    }

    // Try to parse the file
    let content = fs::read_to_string(config_path)
        .map_err(|e| KeyringError::IoError(format!("Failed to read config file: {}", e)))?;

    match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(value) => {
            println!("✅ Configuration file is valid YAML");

            // Check for conflicts
            if let Some(shortcuts) = value.get("shortcuts").and_then(|v| v.as_mapping()) {
                let mut seen = std::collections::HashMap::new();
                let mut has_conflicts = false;

                for (action_key, shortcut_val) in shortcuts {
                    if let Some(shortcut_str) = shortcut_val.as_str() {
                        if let Some(existing_action) = seen.get(shortcut_str) {
                            let action_str = action_key.as_str().unwrap_or("?");
                            println!(
                                "⚠️  Conflict: '{}' is used by both '{}' and '{}'",
                                shortcut_str, existing_action, action_str
                            );
                            has_conflicts = true;
                        } else {
                            seen.insert(
                                shortcut_str.to_string(),
                                action_key.as_str().unwrap_or("?").to_string(),
                            );
                        }
                    }
                }

                if !has_conflicts {
                    println!("✅ No shortcut conflicts detected");
                }
            }

            Ok(())
        }
        Err(e) => Err(KeyringError::InvalidInput {
            context: format!("Invalid YAML: {}", e),
        }),
    }
}

/// Reset keybindings to defaults
fn reset_keybindings(config_path: &Path) -> Result<()> {
    println!("🔄 Resetting keybindings to defaults...");

    // Write default configuration
    fs::write(config_path, crate::tui::keybindings::DEFAULT_KEYBINDINGS)
        .map_err(|e| KeyringError::IoError(format!("Failed to write config: {}", e)))?;

    println!("✅ Keybindings reset to defaults");
    println!("   File: {}", config_path.display());

    Ok(())
}

/// Edit keybindings configuration
fn edit_keybindings(config_path: &Path) -> Result<()> {
    // Ensure default config exists
    if !config_path.exists() {
        fs::write(config_path, crate::tui::keybindings::DEFAULT_KEYBINDINGS)
            .map_err(|e| KeyringError::IoError(format!("Failed to create config: {}", e)))?;
    }

    // Detect editor
    let editor = detect_editor();
    println!("📝 Opening {} with {}...", config_path.display(), editor);

    // Open editor
    let status = Command::new(&editor)
        .arg(config_path)
        .status()
        .map_err(|e| KeyringError::IoError(format!("Failed to open editor: {}", e)))?;

    if !status.success() {
        eprintln!("Warning: Editor exited with non-zero status");
    }

    // Validate after editing
    println!();
    validate_keybindings(config_path)?;

    Ok(())
}

/// Detect the appropriate text editor
fn detect_editor() -> String {
    // Check EDITOR environment variable first
    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.is_empty() {
            return editor;
        }
    }

    // Platform-specific defaults
    #[cfg(target_os = "macos")]
    {
        // Try vim, nvim, code, vi
        for editor in &["vim", "nvim", "code", "vi"] {
            if is_command_available(editor) {
                return editor.to_string();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try vim, nano, nvim, vi
        for editor in &["vim", "nano", "nvim", "vi"] {
            if is_command_available(editor) {
                return editor.to_string();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try code, notepad++, notepad
        for editor in &["code", "notepad++", "notepad"] {
            if is_command_available(editor) {
                return editor.to_string();
            }
        }
    }

    // Fallback
    "vi".to_string()
}

/// Check if a command is available
fn is_command_available(cmd: &str) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("where")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_args_list() {
        use clap::Parser;

        let args = KeybindingsArgs::parse_from(&["ok", "--list"]);
        assert!(args.list);
        assert!(!args.validate);
        assert!(!args.reset);
        assert!(!args.edit);
    }

    #[test]
    fn test_keybindings_args_validate() {
        use clap::Parser;

        let args = KeybindingsArgs::parse_from(&["ok", "--validate"]);
        assert!(args.validate);
        assert!(!args.list);
    }

    #[test]
    fn test_keybindings_args_reset() {
        use clap::Parser;

        let args = KeybindingsArgs::parse_from(&["ok", "--reset"]);
        assert!(args.reset);
        assert!(!args.list);
    }

    #[test]
    fn test_keybindings_args_edit() {
        use clap::Parser;

        let args = KeybindingsArgs::parse_from(&["ok", "--edit"]);
        assert!(args.edit);
        assert!(!args.list);
    }

    #[test]
    fn test_get_config_path() {
        let path = get_config_path();
        assert!(path.ends_with("keybindings.yaml"));
    }

    #[test]
    fn test_detect_editor_fallback() {
        // This will always return at least "vi"
        let editor = detect_editor();
        assert!(!editor.is_empty());
    }
}
