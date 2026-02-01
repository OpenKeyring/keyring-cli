//! MCP Configuration Module
//!
//! This module handles configuration for the MCP (Model Context Protocol) server,
//! including limits for concurrent requests, response sizes, and session caching.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Session cache configuration
///
/// Controls how authorization sessions are cached to avoid repeated
/// authorization prompts for the same operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionCacheConfig {
    /// Maximum number of cached sessions
    pub max_entries: usize,

    /// Time-to-live for cached sessions in seconds
    pub ttl_seconds: u64,
}

impl Default for SessionCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            ttl_seconds: 3600, // 1 hour
        }
    }
}

/// MCP configuration structure
///
/// Contains all configurable limits and settings for the MCP server,
/// including resource limits and caching behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpConfig {
    /// Maximum number of concurrent MCP requests
    pub max_concurrent_requests: usize,

    /// Maximum response size for SSH command execution (bytes)
    pub max_response_size_ssh: usize,

    /// Maximum response size for API tool execution (bytes)
    pub max_response_size_api: usize,

    /// Session cache configuration
    pub session_cache: SessionCacheConfig,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            max_response_size_ssh: 10 * 1024 * 1024, // 10MB
            max_response_size_api: 5 * 1024 * 1024,  // 5MB
            session_cache: SessionCacheConfig::default(),
        }
    }
}

impl McpConfig {
    /// Get the path to the MCP configuration file
    ///
    /// Returns the platform-specific path:
    /// - Linux/macOS: `~/.config/open-keyring/mcp-config.json`
    /// - Windows: `%APPDATA%\open-keyring\mcp-config.json`
    ///
    /// # Returns
    /// The path to the MCP configuration file
    #[must_use]
    pub fn config_path() -> std::path::PathBuf {
        let config_dir = if cfg!(windows) {
            // Windows: %APPDATA%\open-keyring\
            dirs::config_dir()
                .map(|p| p.join("open-keyring"))
                .expect("Failed to determine config directory")
        } else {
            // Linux/macOS: ~/.config/open-keyring/
            dirs::config_dir()
                .map(|p| p.join("open-keyring"))
                .expect("Failed to determine config directory")
        };

        config_dir.join("mcp-config.json")
    }

    /// Load MCP configuration from a file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<McpConfig>` - The loaded configuration or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file contains invalid JSON
    /// - The JSON structure doesn't match McpConfig
    pub fn load(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: McpConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration or create default
    ///
    /// Attempts to load the configuration from the specified path.
    /// If the file doesn't exist or contains invalid data,
    /// creates a new default configuration and saves it.
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// * `Result<McpConfig>` - The loaded or default configuration
    ///
    /// # Errors
    /// Returns an error if:
    /// - The config directory cannot be created
    /// - The configuration file cannot be written
    pub fn load_or_default(path: &Path) -> Result<Self> {
        // Try to load existing config
        if path.exists() {
            match Self::load(path) {
                Ok(config) => return Ok(config),
                Err(_) => {
                    // Invalid config, will create default below
                }
            }
        }

        // Create default config
        let config = Self::default();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::IoError(format!(
                    "Failed to create config directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Save default config
        config.save(path)?;

        Ok(config)
    }

    /// Save MCP configuration to a file
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
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = McpConfig::default();

        assert_eq!(config.max_concurrent_requests, 10);
        assert_eq!(config.max_response_size_ssh, 10 * 1024 * 1024);
        assert_eq!(config.max_response_size_api, 5 * 1024 * 1024);
        assert_eq!(config.session_cache.max_entries, 100);
        assert_eq!(config.session_cache.ttl_seconds, 3600);
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = McpConfig {
            max_concurrent_requests: 20,
            max_response_size_ssh: 20 * 1024 * 1024,
            max_response_size_api: 10 * 1024 * 1024,
            session_cache: SessionCacheConfig {
                max_entries: 200,
                ttl_seconds: 7200,
            },
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: McpConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_session_cache_config_default() {
        let cache_config = SessionCacheConfig::default();

        assert_eq!(cache_config.max_entries, 100);
        assert_eq!(cache_config.ttl_seconds, 3600);
    }
}
