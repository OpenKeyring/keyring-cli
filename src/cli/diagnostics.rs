//! CLI system diagnostics module
//!
//! Provides OpenKeyring system status checking functionality for first-time
//! user detection and configuration diagnostics.

use crate::error::Result;
use std::path::PathBuf;

/// System status category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCategory {
    OK,      // Normal
    Missing, // Missing
    Error,   // Error
}

/// System status item
#[derive(Debug, Clone)]
pub struct StatusItem {
    pub name: &'static str,      // Item name
    pub status: StatusCategory,  // Status
    pub path: Option<PathBuf>,   // Path
    pub message: Option<String>, // Additional message
}

/// System diagnostic result
#[derive(Debug)]
pub struct SystemStatus {
    /// First-time use flag (core: keystore file doesn't exist)
    pub is_first_time: bool,

    /// System health flag (false for first-time, requires all core files for initialized)
    pub is_healthy: bool,

    /// Configuration-related status
    pub config_items: Vec<StatusItem>,

    /// Key-related status
    pub key_items: Vec<StatusItem>,

    /// Data-related status
    pub data_items: Vec<StatusItem>,
}

impl SystemStatus {
    /// Check if first-time use (keystore.json doesn't exist)
    pub fn is_first_time(&self) -> bool {
        self.is_first_time
    }

    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }
}

/// Check system status
///
/// Checks config, key, and data components, returns system status diagnostic result.
/// First-time use criterion: keystore.json doesn't exist.
pub fn check_system_status() -> Result<SystemStatus> {
    todo!("Implement check_system_status")
}

/// Print first-time welcome message
pub fn print_first_time_message() {
    todo!("Implement print_first_time_message")
}

/// Print diagnostic report (non-first-time but with issues)
pub fn print_diagnostic_report(_status: &SystemStatus) {
    todo!("Implement print_diagnostic_report")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_item_creation() {
        let item = StatusItem {
            name: "test_item",
            status: StatusCategory::OK,
            path: Some(PathBuf::from("/tmp/test")),
            message: None,
        };
        assert_eq!(item.name, "test_item");
        assert_eq!(item.status, StatusCategory::OK);
    }

    #[test]
    fn test_status_category_eq() {
        assert_eq!(StatusCategory::OK, StatusCategory::OK);
        assert_eq!(StatusCategory::Missing, StatusCategory::Missing);
        assert_eq!(StatusCategory::Error, StatusCategory::Error);
        assert_ne!(StatusCategory::OK, StatusCategory::Error);
    }

    #[test]
    fn test_status_category_copy() {
        let status1 = StatusCategory::OK;
        let status2 = status1; // Copy should work
        assert_eq!(status1, StatusCategory::OK);
        assert_eq!(status2, StatusCategory::OK);
    }

    #[test]
    fn test_status_item_with_all_fields() {
        let item = StatusItem {
            name: "config_file",
            status: StatusCategory::Missing,
            path: Some(PathBuf::from("/tmp/config.json")),
            message: Some(String::from("Configuration file not found")),
        };
        assert_eq!(item.name, "config_file");
        assert_eq!(item.status, StatusCategory::Missing);
        assert_eq!(item.path, Some(PathBuf::from("/tmp/config.json")));
        assert_eq!(
            item.message,
            Some(String::from("Configuration file not found"))
        );
    }

    #[test]
    fn test_status_item_clone() {
        let item1 = StatusItem {
            name: "keystore",
            status: StatusCategory::OK,
            path: Some(PathBuf::from("/tmp/keystore.json")),
            message: None,
        };
        let item2 = item1.clone();
        assert_eq!(item1.name, item2.name);
        assert_eq!(item1.status, item2.status);
    }

    #[test]
    fn test_system_status_is_first_time() {
        let status = SystemStatus {
            is_first_time: true,
            is_healthy: false,
            config_items: vec![],
            key_items: vec![],
            data_items: vec![],
        };
        assert!(status.is_first_time());
        assert!(!status.is_healthy());
    }

    #[test]
    fn test_system_status_is_healthy() {
        let status = SystemStatus {
            is_first_time: false,
            is_healthy: true,
            config_items: vec![],
            key_items: vec![],
            data_items: vec![],
        };
        assert!(!status.is_first_time());
        assert!(status.is_healthy());
    }

    #[test]
    fn test_system_status_with_items() {
        let config_item = StatusItem {
            name: "config",
            status: StatusCategory::OK,
            path: Some(PathBuf::from("/tmp/config")),
            message: None,
        };
        let status = SystemStatus {
            is_first_time: false,
            is_healthy: true,
            config_items: vec![config_item],
            key_items: vec![],
            data_items: vec![],
        };
        assert_eq!(status.config_items.len(), 1);
        assert_eq!(status.config_items[0].name, "config");
    }
}
