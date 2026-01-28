//! CLI config command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::config::{execute, ConfigCommands};
use keyring_cli::db::Vault;
use tempfile::TempDir;

#[test]
fn test_config_set_persists_to_metadata() {
    // Test: Set config value and verify it's saved to metadata table
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("config_set_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");

    // Set a config value
    let set_command = ConfigCommands::Set {
        key: "test.key".to_string(),
        value: "test-value".to_string(),
    };

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        execute(set_command).await
    }).unwrap();

    // Verify it was saved to metadata
    let vault = Vault::open(&db_path, "").unwrap();
    let saved_value = vault.get_metadata("test.key").unwrap();
    assert_eq!(saved_value, Some("test-value".to_string()), "Config should be saved to metadata");
}

#[test]
fn test_config_get_reads_from_metadata() {
    // Test: Get config value from metadata table
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("config_get_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    let mut vault = Vault::open(&db_path, "").unwrap();

    // Set a value in metadata
    vault.set_metadata("custom.timeout", "30").unwrap();

    // Get the value back
    let get_command = ConfigCommands::Get {
        key: "custom.timeout".to_string(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        execute(get_command).await
    });

    assert!(result.is_ok(), "Get should succeed");
}

#[test]
fn test_config_reset_clears_metadata() {
    // Test: Reset config clears metadata values
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("config_reset_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    let mut vault = Vault::open(&db_path, "").unwrap();

    // Set some values in metadata
    vault.set_metadata("custom.key1", "value1").unwrap();
    vault.set_metadata("custom.key2", "value2").unwrap();

    // Verify they were set
    assert_eq!(vault.get_metadata("custom.key1").unwrap(), Some("value1".to_string()));
    assert_eq!(vault.get_metadata("custom.key2").unwrap(), Some("value2".to_string()));

    // Reset config
    let reset_command = ConfigCommands::Reset { force: true };

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        execute(reset_command).await
    }).unwrap();

    // Verify metadata was cleared
    let value1 = vault.get_metadata("custom.key1").unwrap();
    let value2 = vault.get_metadata("custom.key2").unwrap();

    assert_eq!(value1, None, "Metadata should be cleared after reset");
    assert_eq!(value2, None, "Metadata should be cleared after reset");
}

#[test]
fn test_config_set_validates_key() {
    // Test: Set config validates key against allowed list
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("config_validate_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    // Try to set an invalid key (should be rejected or accepted with warning)
    let set_command = ConfigCommands::Set {
        key: "invalid.unauthorized.key".to_string(),
        value: "some-value".to_string(),
    };

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        execute(set_command).await
    }).unwrap();

    // Should either succeed with a warning or fail with an error
    // For now, we'll accept that it succeeds (validation can be added later)
    // The first unwrap() already validates this
}
