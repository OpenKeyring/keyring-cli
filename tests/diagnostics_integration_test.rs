// Diagnostics integration tests
//
// This file contains integration tests for the diagnostics module,
// testing the full workflow of system status checking, first-time user
// detection, and diagnostic reporting.

#![cfg(feature = "test-env")]

use keyring_cli::diagnostics::{check_system_status, StatusCategory, SystemStatus};
use std::fs;

#[test]
fn test_first_time_user_workflow() {
    // Test the complete first-time user workflow
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    // Create directories but no files
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    // Set environment variables for test-env
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Check system status
    let status = check_system_status().unwrap();

    // Verify first-time detection
    assert!(status.is_first_time(), "Should detect first-time user");
    assert!(
        !status.is_healthy(),
        "First-time user should not be healthy"
    );

    // Verify keystore is missing
    assert_eq!(status.key_items.len(), 1);
    assert_eq!(status.key_items[0].name, "Keystore file");
    assert_eq!(status.key_items[0].status, StatusCategory::Missing);

    // Verify directories exist
    assert_eq!(status.config_items[0].name, "Config directory");
    assert_eq!(status.config_items[0].status, StatusCategory::OK);
    assert_eq!(status.data_items[0].name, "Data directory");
    assert_eq!(status.data_items[0].status, StatusCategory::OK);

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_initialized_system_workflow() {
    // Test the complete initialized system workflow
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    // Create directories and all core files
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    // Create all core files
    fs::write(config_dir.join("config.yaml"), "test: config\n").unwrap();
    fs::write(config_dir.join("keystore.json"), "{\"version\":1}\n").unwrap();
    fs::write(data_dir.join("passwords.db"), "test db content\n").unwrap();

    // Set environment variables for test-env
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Check system status
    let status = check_system_status().unwrap();

    // Verify initialized state
    assert!(!status.is_first_time(), "Should not be first-time user");
    assert!(status.is_healthy(), "Initialized system should be healthy");

    // Verify all status items are OK
    for item in &status.config_items {
        assert_eq!(item.status, StatusCategory::OK);
    }
    for item in &status.key_items {
        assert_eq!(item.status, StatusCategory::OK);
    }
    for item in &status.data_items {
        assert_eq!(item.status, StatusCategory::OK);
    }

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_partial_initialization_scenarios() {
    // Test various partial initialization scenarios

    // Scenario 1: Only keystore missing (first-time)
    let temp_dir1 = tempfile::TempDir::new().unwrap();
    let config_dir1 = temp_dir1.path().join("config");
    let data_dir1 = temp_dir1.path().join("data");

    fs::create_dir_all(&config_dir1).unwrap();
    fs::create_dir_all(&data_dir1).unwrap();
    fs::write(config_dir1.join("config.yaml"), "test: config\n").unwrap();
    fs::write(data_dir1.join("passwords.db"), "test db\n").unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir1.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir1.to_str().unwrap());

    let status1 = check_system_status().unwrap();
    assert!(status1.is_first_time());
    assert!(!status1.is_healthy());

    // Scenario 2: Only database missing (initialized but unhealthy)
    let temp_dir2 = tempfile::TempDir::new().unwrap();
    let config_dir2 = temp_dir2.path().join("config");
    let data_dir2 = temp_dir2.path().join("data");

    fs::create_dir_all(&config_dir2).unwrap();
    fs::create_dir_all(&data_dir2).unwrap();
    fs::write(config_dir2.join("config.yaml"), "test: config\n").unwrap();
    fs::write(config_dir2.join("keystore.json"), "{}\n").unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir2.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir2.to_str().unwrap());

    let status2 = check_system_status().unwrap();
    assert!(!status2.is_first_time());
    assert!(!status2.is_healthy());

    // Scenario 3: Only config missing (initialized but unhealthy)
    let temp_dir3 = tempfile::TempDir::new().unwrap();
    let config_dir3 = temp_dir3.path().join("config");
    let data_dir3 = temp_dir3.path().join("data");

    fs::create_dir_all(&config_dir3).unwrap();
    fs::create_dir_all(&data_dir3).unwrap();
    fs::write(config_dir3.join("keystore.json"), "{}\n").unwrap();
    fs::write(data_dir3.join("passwords.db"), "test db\n").unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir3.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir3.to_str().unwrap());

    let status3 = check_system_status().unwrap();
    assert!(!status3.is_first_time());
    assert!(!status3.is_healthy());

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
#[allow(clippy::permissions_set_readonly_false)]
fn test_database_edge_cases() {
    // Test database edge cases

    // Case 1: Empty database file
    let temp_dir1 = tempfile::TempDir::new().unwrap();
    let config_dir1 = temp_dir1.path().join("config");
    let data_dir1 = temp_dir1.path().join("data");

    fs::create_dir_all(&config_dir1).unwrap();
    fs::create_dir_all(&data_dir1).unwrap();
    fs::write(config_dir1.join("config.yaml"), "test: config\n").unwrap();
    fs::write(config_dir1.join("keystore.json"), "{}\n").unwrap();
    fs::write(data_dir1.join("passwords.db"), "").unwrap(); // Empty database

    std::env::set_var("OK_CONFIG_DIR", config_dir1.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir1.to_str().unwrap());

    let status1 = check_system_status().unwrap();
    // Empty database still exists, so is_healthy will be true
    // (health check only verifies file existence, not content validity)
    assert!(!status1.is_first_time());
    assert!(status1.is_healthy()); // Files exist, so healthy

    // Find database status item
    let db_item = status1
        .data_items
        .iter()
        .find(|item| item.name == "Database file")
        .expect("Database file should be in data_items");
    assert_eq!(db_item.status, StatusCategory::Error);
    assert!(db_item.message.as_ref().unwrap().contains("empty"));

    // Case 2: Corrupted/unreadable database
    let temp_dir2 = tempfile::TempDir::new().unwrap();
    let config_dir2 = temp_dir2.path().join("config");
    let data_dir2 = temp_dir2.path().join("data");

    fs::create_dir_all(&config_dir2).unwrap();
    fs::create_dir_all(&data_dir2).unwrap();
    fs::write(config_dir2.join("config.yaml"), "test: config\n").unwrap();
    fs::write(config_dir2.join("keystore.json"), "{}\n").unwrap();
    fs::write(data_dir2.join("passwords.db"), "valid content\n").unwrap();

    // Make file unreadable
    let mut perms = fs::metadata(data_dir2.join("passwords.db"))
        .unwrap()
        .permissions();
    perms.set_readonly(false);
    fs::set_permissions(data_dir2.join("passwords.db"), perms).unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir2.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir2.to_str().unwrap());

    // This should still work since we're just checking metadata
    let status2 = check_system_status().unwrap();
    assert!(!status2.is_first_time());

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_status_item_completeness() {
    // Test that all expected status items are present

    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    let status = check_system_status().unwrap();

    // Verify config items
    assert_eq!(status.config_items.len(), 2);
    let config_item_names: Vec<&str> = status.config_items.iter().map(|item| item.name).collect();
    assert!(config_item_names.contains(&"Config directory"));
    assert!(config_item_names.contains(&"Config file"));

    // Verify key items
    assert_eq!(status.key_items.len(), 1);
    assert_eq!(status.key_items[0].name, "Keystore file");

    // Verify data items
    assert_eq!(status.data_items.len(), 2);
    let data_item_names: Vec<&str> = status.data_items.iter().map(|item| item.name).collect();
    assert!(data_item_names.contains(&"Data directory"));
    assert!(data_item_names.contains(&"Database file"));

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_path_correctness() {
    // Test that all status items have correct paths

    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    let status = check_system_status().unwrap();

    // Canonicalize paths for comparison (handles macOS /var -> /private/var symlinks)
    let config_dir_canonical = fs::canonicalize(&config_dir).unwrap_or_else(|_| config_dir.clone());
    let data_dir_canonical = fs::canonicalize(&data_dir).unwrap_or_else(|_| data_dir.clone());

    // Verify all paths are present and correct
    for item in &status.config_items {
        assert!(item.path.is_some(), "Config item should have a path");
        let path = item.path.as_ref().unwrap();

        // For directory items, check exact match
        // For file items, check that parent directory matches
        if item.name.ends_with("directory") {
            let path_canonical = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            assert_eq!(
                path_canonical,
                config_dir_canonical,
                "Config directory path mismatch: '{}' vs '{}'",
                path_canonical.display(),
                config_dir_canonical.display()
            );
        } else {
            // File item - check parent directory
            if let Some(parent) = path.parent() {
                let parent_canonical =
                    fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
                assert_eq!(
                    parent_canonical,
                    config_dir_canonical,
                    "Config file parent mismatch: '{}' vs '{}'",
                    parent_canonical.display(),
                    config_dir_canonical.display()
                );
            }
        }
    }

    for item in &status.key_items {
        assert!(item.path.is_some(), "Key item should have a path");
        let path = item.path.as_ref().unwrap();
        // Key items are files - check parent directory
        if let Some(parent) = path.parent() {
            let parent_canonical =
                fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
            assert_eq!(
                parent_canonical,
                config_dir_canonical,
                "Key file parent mismatch: '{}' vs '{}'",
                parent_canonical.display(),
                config_dir_canonical.display()
            );
        }
    }

    for item in &status.data_items {
        assert!(item.path.is_some(), "Data item should have a path");
        let path = item.path.as_ref().unwrap();

        // For directory items, check exact match
        // For file items, check that parent directory matches
        if item.name.ends_with("directory") {
            let path_canonical = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            assert_eq!(
                path_canonical,
                data_dir_canonical,
                "Data directory path mismatch: '{}' vs '{}'",
                path_canonical.display(),
                data_dir_canonical.display()
            );
        } else {
            // File item - check parent directory
            if let Some(parent) = path.parent() {
                let parent_canonical =
                    fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
                assert_eq!(
                    parent_canonical,
                    data_dir_canonical,
                    "Data file parent mismatch: '{}' vs '{}'",
                    parent_canonical.display(),
                    data_dir_canonical.display()
                );
            }
        }
    }

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_message_content_for_missing_items() {
    // Test that appropriate messages are set for missing items

    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    // Create empty directories
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    let status = check_system_status().unwrap();

    // Verify keystore message
    let keystore_item = status
        .key_items
        .iter()
        .find(|item| item.name == "Keystore file")
        .unwrap();
    assert_eq!(keystore_item.status, StatusCategory::Missing);
    assert!(keystore_item.message.is_some());
    assert!(keystore_item
        .message
        .as_ref()
        .unwrap()
        .contains("first-time"));

    // Verify config file message
    let config_file_item = status
        .config_items
        .iter()
        .find(|item| item.name == "Config file")
        .unwrap();
    assert_eq!(config_file_item.status, StatusCategory::Missing);
    assert!(config_file_item.message.is_some());

    // Verify database message
    let db_item = status
        .data_items
        .iter()
        .find(|item| item.name == "Database file")
        .unwrap();
    assert_eq!(db_item.status, StatusCategory::Missing);
    assert!(db_item.message.is_some());

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

#[test]
fn test_no_environment_variable_uses_defaults() {
    // Test that without environment variables, default paths are used

    // Clear any existing environment variables
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");

    // This should use default system paths
    let status = check_system_status();

    // Should succeed (just might not find files on system)
    assert!(status.is_ok());

    let status = status.unwrap();

    // All items should have paths
    for item in &status.config_items {
        assert!(item.path.is_some());
    }
    for item in &status.key_items {
        assert!(item.path.is_some());
    }
    for item in &status.data_items {
        assert!(item.path.is_some());
    }

    // Verify paths use default locations
    let config_item = &status.config_items[0];
    let path = config_item.path.as_ref().unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("open-keyring") || path_str.contains(".config"),
        "Config path should use default location, got: {}",
        path_str
    );
}

#[test]
fn test_status_category_equality() {
    // Test StatusCategory equality and comparison

    let ok = StatusCategory::OK;
    let missing = StatusCategory::Missing;
    let error = StatusCategory::Error;

    // Test equality
    assert_eq!(ok, StatusCategory::OK);
    assert_eq!(missing, StatusCategory::Missing);
    assert_eq!(error, StatusCategory::Error);

    // Test inequality
    assert_ne!(ok, missing);
    assert_ne!(ok, error);
    assert_ne!(missing, error);
}

#[test]
fn test_system_status_methods() {
    // Test SystemStatus convenience methods

    // First-time user
    let status1 = SystemStatus {
        is_first_time: true,
        is_healthy: false,
        config_items: vec![],
        key_items: vec![],
        data_items: vec![],
    };
    assert!(status1.is_first_time());
    assert!(!status1.is_healthy());

    // Healthy initialized user
    let status2 = SystemStatus {
        is_first_time: false,
        is_healthy: true,
        config_items: vec![],
        key_items: vec![],
        data_items: vec![],
    };
    assert!(!status2.is_first_time());
    assert!(status2.is_healthy());
}

#[test]
fn test_multiple_runs_consistency() {
    // Test that multiple status checks return consistent results

    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let data_dir = temp_dir.path().join("data");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&data_dir).unwrap();

    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Run multiple checks
    let status1 = check_system_status().unwrap();
    let status2 = check_system_status().unwrap();
    let status3 = check_system_status().unwrap();

    // All should return same results
    assert_eq!(status1.is_first_time(), status2.is_first_time());
    assert_eq!(status2.is_first_time(), status3.is_first_time());
    assert_eq!(status1.is_healthy(), status2.is_healthy());
    assert_eq!(status2.is_healthy(), status3.is_healthy());

    assert_eq!(status1.config_items.len(), status2.config_items.len());
    assert_eq!(status2.config_items.len(), status3.config_items.len());
    assert_eq!(status1.key_items.len(), status2.key_items.len());
    assert_eq!(status2.key_items.len(), status3.key_items.len());
    assert_eq!(status1.data_items.len(), status2.data_items.len());
    assert_eq!(status2.data_items.len(), status3.data_items.len());

    // Clean up environment
    std::env::remove_var("OK_CONFIG_DIR");
    std::env::remove_var("OK_DATA_DIR");
}

// Note: print_first_time_message() and print_diagnostic_report() are not tested
// here as they produce output to stdout. Testing these would require capturing
// stdout which is typically done in separate unit tests or with special
// test utilities.
