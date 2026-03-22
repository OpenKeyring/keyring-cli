//! Type definitions for ProviderConfigScreen
//!
//! Contains ConfigField, ProviderConfig, and related types.

use crate::cloud::CloudProvider;
use std::collections::HashMap;

/// A single configuration field
#[derive(Debug, Clone)]
pub struct ConfigField {
    /// Field label (e.g., "WebDAV URL")
    pub label: String,
    /// Current field value
    pub value: String,
    /// Whether this is a password field (masked display)
    pub is_password: bool,
    /// Whether this field currently has focus
    pub is_focused: bool,
}

impl ConfigField {
    /// Creates a new configuration field
    pub fn new(label: &str, is_password: bool) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            is_password,
            is_focused: false,
        }
    }
}

/// Provider configuration data
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Cloud provider type
    pub provider: CloudProvider,
    /// Configuration values keyed by field name
    pub values: HashMap<String, String>,
}

impl ProviderConfig {
    /// Creates a new provider configuration
    pub fn new(provider: CloudProvider) -> Self {
        Self {
            provider,
            values: HashMap::new(),
        }
    }

    /// Sets a configuration value
    pub fn set(&mut self, key: &str, value: String) {
        self.values.insert(key.to_string(), value);
    }

    /// Gets a configuration value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
}
