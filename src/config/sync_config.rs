//! Sync Configuration File Management
//!
//! This module provides configuration file management for sync settings,
//! using YAML serialization for human-readable configuration.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Sync configuration file structure
///
/// This configuration controls how the sync feature operates,
/// including which provider to use and sync behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncConfigFile {
    /// Whether sync is enabled
    pub sync_enabled: bool,

    /// Cloud storage provider (icloud, dropbox, google_drive, webdav, sftp)
    pub provider: String,

    /// Optional custom path for iCloud Drive
    pub icloud_path: Option<String>,

    /// Debounce delay in seconds before triggering sync after file changes
    pub debounce_delay: u64,

    /// Whether to automatically sync after changes
    pub auto_sync: bool,
}

impl Default for SyncConfigFile {
    fn default() -> Self {
        Self {
            sync_enabled: false,
            provider: "icloud".to_string(),
            icloud_path: None,
            debounce_delay: 5,
            auto_sync: false,
        }
    }
}

impl SyncConfigFile {
    /// Load sync configuration from a YAML file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<SyncConfigFile>` - The loaded configuration or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file contains invalid YAML
    /// - The YAML structure doesn't match SyncConfigFile
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save sync configuration to a YAML file
    ///
    /// # Arguments
    /// * `path` - Path where the configuration file should be saved
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be created or written
    /// - The parent directory doesn't exist
    /// - Serialization fails
    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = serde_yaml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = SyncConfigFile::default();
        assert!(!config.sync_enabled);
        assert_eq!(config.provider, "icloud");
        assert_eq!(config.icloud_path, None);
        assert_eq!(config.debounce_delay, 5);
        assert!(!config.auto_sync);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = SyncConfigFile {
            sync_enabled: true,
            provider: "dropbox".to_string(),
            icloud_path: Some("~/Dropbox/open-keyring".to_string()),
            debounce_delay: 10,
            auto_sync: true,
        };

        let yaml = serde_yaml::to_string(&original).unwrap();
        let deserialized: SyncConfigFile = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(original, deserialized);
    }
}
