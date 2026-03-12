//! TUI Configuration Module
//!
//! Provides configuration persistence for TUI settings including
//! clipboard timeout, trash retention, and password policy defaults.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::io;

/// Main TUI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Clipboard timeout in seconds (default: 30)
    pub clipboard_timeout_seconds: u64,
    /// Trash retention period in days (default: 30)
    pub trash_retention_days: u32,
    /// Default password policy
    pub default_password_policy: PasswordPolicyConfig,
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Auto-lock timeout in seconds (0 = disabled)
    pub auto_lock_seconds: u64,
    /// Show password strength indicator
    pub show_password_strength: bool,
    /// Confirm before delete
    pub confirm_delete: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            clipboard_timeout_seconds: 30,
            trash_retention_days: 30,
            default_password_policy: PasswordPolicyConfig::default(),
            theme: ThemeConfig::default(),
            auto_lock_seconds: 300, // 5 minutes
            show_password_strength: true,
            confirm_delete: true,
        }
    }
}

impl TuiConfig {
    /// Create new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the configuration file path
    pub fn config_path(config_dir: &Path) -> PathBuf {
        config_dir.join("tui_config.json")
    }

    /// Load configuration from a directory
    pub fn load(config_dir: &Path) -> io::Result<Self> {
        let config_file = Self::config_path(config_dir);

        if !config_file.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_file)?;

        // Parse JSON, falling back to default for parse errors
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    /// Save configuration to a directory
    pub fn save(&self, config_dir: &Path) -> io::Result<()> {
        std::fs::create_dir_all(config_dir)?;

        let config_file = Self::config_path(config_dir);
        let content = serde_json::to_string_pretty(self)?;

        std::fs::write(&config_file, content)
    }

    /// Set clipboard timeout
    #[must_use]
    pub fn with_clipboard_timeout(mut self, seconds: u64) -> Self {
        self.clipboard_timeout_seconds = seconds;
        self
    }

    /// Set trash retention days
    #[must_use]
    pub fn with_trash_retention(mut self, days: u32) -> Self {
        self.trash_retention_days = days;
        self
    }

    /// Set auto-lock timeout
    #[must_use]
    pub fn with_auto_lock(mut self, seconds: u64) -> Self {
        self.auto_lock_seconds = seconds;
        self
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    /// Default password length
    pub length: u8,
    /// Minimum number of digits
    pub min_digits: u8,
    /// Minimum number of special characters
    pub min_special: u8,
    /// Minimum number of lowercase letters
    pub min_lowercase: u8,
    /// Minimum number of uppercase letters
    pub min_uppercase: u8,
    /// Password type preference
    pub password_type: PasswordTypeConfig,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            length: 16,
            min_digits: 2,
            min_special: 1,
            min_lowercase: 1,
            min_uppercase: 1,
            password_type: PasswordTypeConfig::Random,
        }
    }
}

/// Password type configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasswordTypeConfig {
    /// Random password with mixed characters
    Random,
    /// Memorable word-based password
    Memorable,
    /// Numeric PIN
    Pin,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme variant
    pub variant: ThemeVariant,
    /// Show colorful icons
    pub colorful_icons: bool,
    /// Compact mode (less spacing)
    pub compact_mode: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            variant: ThemeVariant::Dark,
            colorful_icons: true,
            compact_mode: false,
        }
    }
}

/// Theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeVariant {
    /// Dark theme (default)
    Dark,
    /// Light theme
    Light,
    /// High contrast theme
    HighContrast,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = TuiConfig::default();

        assert_eq!(config.clipboard_timeout_seconds, 30);
        assert_eq!(config.trash_retention_days, 30);
        assert_eq!(config.auto_lock_seconds, 300);
        assert!(config.show_password_strength);
        assert!(config.confirm_delete);
    }

    #[test]
    fn test_config_builder() {
        let config = TuiConfig::new()
            .with_clipboard_timeout(60)
            .with_trash_retention(7)
            .with_auto_lock(600);

        assert_eq!(config.clipboard_timeout_seconds, 60);
        assert_eq!(config.trash_retention_days, 7);
        assert_eq!(config.auto_lock_seconds, 600);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let original = TuiConfig::new()
            .with_clipboard_timeout(45)
            .with_trash_retention(14);

        // Save
        original.save(config_path).unwrap();

        // Load
        let loaded = TuiConfig::load(config_path).unwrap();

        assert_eq!(loaded.clipboard_timeout_seconds, 45);
        assert_eq!(loaded.trash_retention_days, 14);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = TuiConfig::load(temp_dir.path()).unwrap();

        // Should return default
        assert_eq!(config.clipboard_timeout_seconds, 30);
    }

    #[test]
    fn test_config_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = TuiConfig::config_path(temp_dir.path());

        // Write invalid JSON
        std::fs::write(&config_file, "{ invalid }").unwrap();

        let config = TuiConfig::load(temp_dir.path()).unwrap();

        // Should return default on parse error
        assert_eq!(config.clipboard_timeout_seconds, 30);
    }

    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicyConfig::default();

        assert_eq!(policy.length, 16);
        assert_eq!(policy.min_digits, 2);
        assert_eq!(policy.min_special, 1);
        assert_eq!(policy.min_lowercase, 1);
        assert_eq!(policy.min_uppercase, 1);
        assert_eq!(policy.password_type, PasswordTypeConfig::Random);
    }

    #[test]
    fn test_theme_config_default() {
        let theme = ThemeConfig::default();

        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert!(theme.colorful_icons);
        assert!(!theme.compact_mode);
    }

    #[test]
    fn test_config_serialization() {
        let config = TuiConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: TuiConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.clipboard_timeout_seconds, config.clipboard_timeout_seconds);
    }
}
