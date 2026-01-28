//! CLI config command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::config::{execute, ConfigCommands};
use keyring_cli::db::Vault;
use tempfile::TempDir;

/// Helper to set up test environment and clean up afterwards
struct TestEnv {
    _temp_dir: TempDir,
    db_path: std::path::PathBuf,
}

impl TestEnv {
    fn setup(test_name: &str) -> Self {
        // Clean up any existing environment variables first
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(format!("config_{}", test_name));
        let data_dir = temp_dir.path().join(format!("data_{}", test_name));
        std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
        std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
        std::env::set_var("OK_MASTER_PASSWORD", "test-password");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        let db_path = data_dir.join("passwords.db");

        Self { _temp_dir: temp_dir, db_path }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Clean up environment variables
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");
    }
}

#[test]
fn test_config_set_persists_to_metadata() {
    let _env = TestEnv::setup("set_persists");

    // Set a config value
    let set_command = ConfigCommands::Set {
        key: "clipboard.timeout".to_string(),
        value: "45".to_string(),
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { execute(set_command).await })
        .unwrap();

    // Give time for WAL to checkpoint and for all connections to close
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Drop the vault from execute() before opening a new one
    // Verify it was saved to metadata
    let vault = Vault::open(&_env.db_path, "").unwrap();
    let saved_value = vault.get_metadata("clipboard.timeout").unwrap();
    assert_eq!(
        saved_value,
        Some("45".to_string()),
        "Config should be saved to metadata: got {:?}",
        saved_value
    );
}

#[test]
fn test_config_get_reads_from_metadata() {
    let _env = TestEnv::setup("get_reads");

    // Set a value in metadata
    {
        let mut vault = Vault::open(&_env.db_path, "").unwrap();
        vault.set_metadata("custom.timeout", "30").unwrap();
    }

    // Give time for WAL to checkpoint
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Get the value back
    let get_command = ConfigCommands::Get {
        key: "custom.timeout".to_string(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { execute(get_command).await });

    assert!(result.is_ok(), "Get should succeed: {:?}", result.err());
}

#[test]
fn test_config_reset_clears_custom_metadata() {
    let _env = TestEnv::setup("reset_clears");

    // Set custom values directly in metadata
    {
        let mut vault = Vault::open(&_env.db_path, "").unwrap();
        vault.set_metadata("custom.key1", "value1").unwrap();
        vault.set_metadata("custom.key2", "value2").unwrap();
    }

    // Give time for WAL to checkpoint
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Verify they were set
    let vault = Vault::open(&_env.db_path, "").unwrap();
    assert_eq!(
        vault.get_metadata("custom.key1").unwrap(),
        Some("value1".to_string())
    );
    assert_eq!(
        vault.get_metadata("custom.key2").unwrap(),
        Some("value2".to_string())
    );

    // Close vault to release lock
    drop(vault);
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Reset config
    let reset_command = ConfigCommands::Reset { force: true };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { execute(reset_command).await })
        .unwrap();

    // Give time for WAL to checkpoint and for all connections to close
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify custom metadata was cleared
    let vault = Vault::open(&_env.db_path, "").unwrap();
    let value1 = vault.get_metadata("custom.key1").unwrap();
    let value2 = vault.get_metadata("custom.key2").unwrap();

    assert_eq!(value1, None, "Custom metadata should be cleared after reset, got {:?}", value1);
    assert_eq!(value2, None, "Custom metadata should be cleared after reset, got {:?}", value2);
}

#[test]
fn test_config_set_validates_key() {
    let _env = TestEnv::setup("validates_key");

    // Try to set an invalid key (should be rejected)
    let set_command = ConfigCommands::Set {
        key: "invalid.unauthorized.key".to_string(),
        value: "some-value".to_string(),
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { execute(set_command).await });

    assert!(result.is_err(), "Should reject invalid configuration key");
}
