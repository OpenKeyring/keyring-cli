//! CLI system diagnostics module
//!
//! Provides OpenKeyring system status checking functionality for first-time
//! user detection and configuration diagnostics.

use crate::error::Result;
use std::path::PathBuf;

/// Get config directory, with test-env support if feature is enabled
#[cfg(feature = "test-env")]
fn get_config_dir() -> PathBuf {
    if let Ok(config_dir) = std::env::var("OK_CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        get_default_config_dir()
    }
}

/// Get data directory, with test-env support if feature is enabled
#[cfg(feature = "test-env")]
fn get_data_dir() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OK_DATA_DIR") {
        PathBuf::from(data_dir)
    } else {
        get_default_data_dir()
    }
}

/// Get default config directory (no test-env override)
fn get_default_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("open-keyring"))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config").join("open-keyring")
        })
}

/// Get default data directory (no test-env override)
fn get_default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|p| p.join("open-keyring"))
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("open-keyring")
        })
}

/// Get config directory without test-env support
#[cfg(not(feature = "test-env"))]
fn get_config_dir() -> PathBuf {
    get_default_config_dir()
}

/// Get data directory without test-env support
#[cfg(not(feature = "test-env"))]
fn get_data_dir() -> PathBuf {
    get_default_data_dir()
}

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

/// Check system status with custom paths (for testing)
///
/// Checks config, key, and data components, returns system status diagnostic result.
/// First-time use criterion: keystore.json doesn't exist.
pub fn check_system_status() -> Result<SystemStatus> {
    check_system_status_with_dirs(get_config_dir(), get_data_dir())
}

/// Check system status with specific directories (for testing)
///
/// This function performs the actual status checks given config and data directories.
fn check_system_status_with_dirs(config_dir: PathBuf, data_dir: PathBuf) -> Result<SystemStatus> {
    use std::fs;

    // Check Config directory status
    let config_dir_status = if config_dir.exists() {
        StatusItem {
            name: "Config directory",
            status: StatusCategory::OK,
            path: Some(config_dir.clone()),
            message: None,
        }
    } else {
        StatusItem {
            name: "Config directory",
            status: StatusCategory::Missing,
            path: Some(config_dir.clone()),
            message: Some("Config directory not found".to_string()),
        }
    };

    // Check Data directory status
    let data_dir_status = if data_dir.exists() {
        StatusItem {
            name: "Data directory",
            status: StatusCategory::OK,
            path: Some(data_dir.clone()),
            message: None,
        }
    } else {
        StatusItem {
            name: "Data directory",
            status: StatusCategory::Missing,
            path: Some(data_dir.clone()),
            message: Some("Data directory not found".to_string()),
        }
    };

    let config_file = config_dir.join("config.yaml");
    let keystore_file = config_dir.join("keystore.json");
    let database_file = data_dir.join("passwords.db");

    // Check config file
    let config_status = if config_file.exists() {
        StatusItem {
            name: "Config file",
            status: StatusCategory::OK,
            path: Some(config_file.clone()),
            message: None,
        }
    } else {
        StatusItem {
            name: "Config file",
            status: StatusCategory::Missing,
            path: Some(config_file.clone()),
            message: Some("Configuration file not found".to_string()),
        }
    };

    // Check keystore file (determines first-time use)
    let keystore_status = if keystore_file.exists() {
        StatusItem {
            name: "Keystore file",
            status: StatusCategory::OK,
            path: Some(keystore_file.clone()),
            message: None,
        }
    } else {
        StatusItem {
            name: "Keystore file",
            status: StatusCategory::Missing,
            path: Some(keystore_file.clone()),
            message: Some("Keystore file not found (first-time use)".to_string()),
        }
    };

    // Check database file
    let database_status = if database_file.exists() {
        // Verify database is readable
        match fs::metadata(&database_file) {
            Ok(metadata) => {
                if metadata.len() > 0 {
                    StatusItem {
                        name: "Database file",
                        status: StatusCategory::OK,
                        path: Some(database_file.clone()),
                        message: None,
                    }
                } else {
                    StatusItem {
                        name: "Database file",
                        status: StatusCategory::Error,
                        path: Some(database_file.clone()),
                        message: Some("Database file is empty".to_string()),
                    }
                }
            }
            Err(e) => StatusItem {
                name: "Database file",
                status: StatusCategory::Error,
                path: Some(database_file.clone()),
                message: Some(format!("Cannot read database: {}", e)),
            },
        }
    } else {
        StatusItem {
            name: "Database file",
            status: StatusCategory::Missing,
            path: Some(database_file.clone()),
            message: Some("Database file not found".to_string()),
        }
    };

    // Determine if first-time use (keystore.json doesn't exist)
    let is_first_time = !keystore_file.exists();

    // Determine system health
    // For first-time users, is_healthy = false (needs initialization)
    // For existing users, all core files must be present
    let is_healthy = if is_first_time {
        false
    } else {
        config_file.exists() && database_file.exists()
    };

    Ok(SystemStatus {
        is_first_time,
        is_healthy,
        config_items: vec![config_dir_status, config_status],
        key_items: vec![keystore_status],
        data_items: vec![data_dir_status, database_status],
    })
}

/// Print first-time welcome message
pub fn print_first_time_message() {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("💡 Detected first-time use of OpenKeyring");
    println!();
    println!("Welcome to OpenKeyring! Before you start, you need to");
    println!("complete the initial setup.");
    println!();
    println!("Please run one of the following commands:");
    println!("  • Interactive wizard: ok wizard");
    println!("  • TUI mode:           ok");
    println!("═══════════════════════════════════════════════════");
}

/// Print diagnostic report (non-first-time but with issues)
pub fn print_diagnostic_report(status: &SystemStatus) {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("⚠️  System Status Abnormal");
    println!();
    println!("Detected the following issues with your OpenKeyring configuration:");
    println!();

    // Helper function to format status icon
    fn status_icon(status: &StatusCategory) -> &str {
        match status {
            StatusCategory::OK => "✅",
            StatusCategory::Missing => "❌",
            StatusCategory::Error => "⚠️",
        }
    }

    // Print Configuration section
    println!("  📁 Configuration");
    for item in &status.config_items {
        let icon = status_icon(&item.status);
        let status_text = match item.status {
            StatusCategory::OK => "",
            StatusCategory::Missing => " (missing",
            StatusCategory::Error => " (error",
        };
        let extra = match &item.message {
            Some(msg) => format!(", {}", msg),
            None => match item.status {
                StatusCategory::Missing => ", will auto-generate)".to_string(),
                StatusCategory::Error => ")".to_string(),
                _ => "".to_string(),
            },
        };
        let path_display = item.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "N/A".to_string());
        println!("      {} {}: {}{}", icon, item.name, path_display, status_text);
        if !extra.is_empty() && item.status != StatusCategory::OK {
            println!("{}", extra);
        }
    }

    // Print Keys section
    println!();
    println!("  🔑 Keys");
    for item in &status.key_items {
        let icon = status_icon(&item.status);
        let status_text = match item.status {
            StatusCategory::OK => "",
            StatusCategory::Missing => " (missing)",
            StatusCategory::Error => " (error)",
        };
        let path_display = item.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "N/A".to_string());
        println!("      {} {}: {}{}", icon, item.name, path_display, status_text);
    }

    // Print Data section
    println!();
    println!("  💾 Data");
    for item in &status.data_items {
        let icon = status_icon(&item.status);
        let status_text = match item.status {
            StatusCategory::OK => "",
            StatusCategory::Missing => " (missing)",
            StatusCategory::Error => " (error)",
        };
        let path_display = item.path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "N/A".to_string());
        println!("      {} {}: {}{}", icon, item.name, path_display, status_text);
    }

    println!();
    println!("───────────────────────────────────────────────────");
    println!("💡 Suggested solutions:");
    println!();
    println!("  • Missing keystore: Run 'ok recover' to restore your keystore");
    println!("  • Missing database: Data may be lost, re-run 'ok wizard'");
    println!("  • Missing config:  Will auto-generate default config");
    println!("═══════════════════════════════════════════════════");
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

    #[test]
    fn test_first_time_detection_no_files() {
        // Test first-time scenario: keystore.json doesn't exist
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Create only config file, not keystore
        std::fs::write(config_dir.join("config.yaml"), "test: config").unwrap();

        let result = check_system_status_with_dirs(config_dir.clone(), data_dir.clone());

        assert!(result.is_ok());
        let status = result.unwrap();
        // First-time is determined by keystore.json not existing
        assert!(status.is_first_time());
        assert!(!status.is_healthy());

        // Verify Config directory status
        assert_eq!(status.config_items.len(), 2);
        assert_eq!(status.config_items[0].name, "Config directory");
        assert_eq!(status.config_items[0].status, StatusCategory::OK);

        // Verify Config file status
        assert_eq!(status.config_items[1].name, "Config file");
        assert_eq!(status.config_items[1].status, StatusCategory::OK);

        // Verify Data directory status
        assert_eq!(status.data_items.len(), 2);
        assert_eq!(status.data_items[0].name, "Data directory");
        assert_eq!(status.data_items[0].status, StatusCategory::OK);

        // Verify Keystore file status
        assert_eq!(status.key_items.len(), 1);
        assert_eq!(status.key_items[0].name, "Keystore file");
        assert_eq!(status.key_items[0].status, StatusCategory::Missing);
    }

    #[test]
    fn test_initialized_system_detection() {
        // Test initialized state: all core files exist
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Create all files for initialized state
        std::fs::write(config_dir.join("config.yaml"), "test: config").unwrap();
        std::fs::write(config_dir.join("keystore.json"), "{}").unwrap();
        std::fs::write(data_dir.join("passwords.db"), "test db").unwrap();

        let result = check_system_status_with_dirs(config_dir.clone(), data_dir.clone());

        assert!(result.is_ok());
        let status = result.unwrap();
        // All files exist, not first-time
        assert!(!status.is_first_time());
        // With all core files present, should be healthy
        assert!(status.is_healthy());

        // Verify all status items use the correct names
        assert_eq!(status.config_items[0].name, "Config directory");
        assert_eq!(status.config_items[1].name, "Config file");
        assert_eq!(status.key_items[0].name, "Keystore file");
        assert_eq!(status.data_items[0].name, "Data directory");
        assert_eq!(status.data_items[1].name, "Database file");

        // All items should be OK
        assert_eq!(status.config_items[0].status, StatusCategory::OK);
        assert_eq!(status.config_items[1].status, StatusCategory::OK);
        assert_eq!(status.key_items[0].status, StatusCategory::OK);
        assert_eq!(status.data_items[0].status, StatusCategory::OK);
        assert_eq!(status.data_items[1].status, StatusCategory::OK);
    }

    #[test]
    fn test_missing_keystore_only() {
        // Test scenario where only keystore is missing (config and DB exist)
        // This should still be considered first-time use
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Only create config file and database, not keystore
        std::fs::write(config_dir.join("config.yaml"), "test: config").unwrap();
        std::fs::write(data_dir.join("passwords.db"), "test db").unwrap();

        let result = check_system_status_with_dirs(config_dir.clone(), data_dir.clone());

        assert!(result.is_ok());
        let status = result.unwrap();

        // keystore doesn't exist = first-time use
        assert!(status.is_first_time());
        assert!(!status.is_healthy());

        // Verify keystore status
        let keystore_item = status.key_items.iter()
            .find(|item| item.name == "Keystore file")
            .expect("Keystore file should be in key_items");
        assert_eq!(keystore_item.status, StatusCategory::Missing);

        // Config and database should exist
        assert_eq!(status.config_items[1].status, StatusCategory::OK);
        assert_eq!(status.data_items[1].status, StatusCategory::OK);
    }

    #[test]
    fn test_missing_database_after_init() {
        // Test scenario where keystore exists but database is missing
        // This indicates an initialized system with data issues
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Only create keystore and config, not database
        std::fs::write(config_dir.join("keystore.json"), "{}").unwrap();
        std::fs::write(config_dir.join("config.yaml"), "test: config").unwrap();

        let result = check_system_status_with_dirs(config_dir.clone(), data_dir.clone());

        assert!(result.is_ok());
        let status = result.unwrap();

        // keystore exists = not first-time, but database missing = unhealthy
        assert!(!status.is_first_time());
        assert!(!status.is_healthy());

        // Verify database status
        let db_item = status.data_items.iter()
            .find(|item| item.name == "Database file")
            .expect("Database file should be in data_items");
        assert_eq!(db_item.status, StatusCategory::Missing);

        // Keystore should be OK
        assert_eq!(status.key_items[0].status, StatusCategory::OK);
    }

    #[test]
    fn test_config_auto_generate_scenario() {
        // Test scenario where only config is missing (keystore and DB exist)
        // Note: Even though config can be auto-generated, the system health check
        // requires all core files to be present for an initialized system
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Only create keystore and database, config missing
        std::fs::write(config_dir.join("keystore.json"), "{}").unwrap();
        std::fs::write(data_dir.join("passwords.db"), "test db").unwrap();

        let result = check_system_status_with_dirs(config_dir.clone(), data_dir.clone());

        assert!(result.is_ok());
        let status = result.unwrap();

        // Not first-time (keystore exists), but unhealthy due to missing config
        assert!(!status.is_first_time());
        assert!(!status.is_healthy());

        // Verify config file status is Missing
        let config_item = status.config_items.iter()
            .find(|item| item.name == "Config file")
            .expect("Config file should be in config_items");
        assert_eq!(config_item.status, StatusCategory::Missing);

        // Keystore and database should be OK
        assert_eq!(status.key_items[0].status, StatusCategory::OK);
        assert_eq!(status.data_items[1].status, StatusCategory::OK);
    }
}
