//! CLI devices command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::devices::{manage_devices, DevicesArgs};
use keyring_cli::db::vault::Vault;
use serial_test::serial;
use tempfile::TempDir;

/// Helper to set up test environment
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

        Self {
            _temp_dir: temp_dir,
            db_path,
        }
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

#[serial]
#[test]
fn test_devices_command_list_with_no_devices() {
    let env = TestEnv::setup("list_no_devices");

    // Create vault
    {
        let _vault = Vault::open(&env.db_path, "").unwrap();
        // No devices registered
    }

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(200));

    // List devices
    let args = DevicesArgs { remove: None };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { manage_devices(args).await });

    assert!(result.is_ok(), "List should succeed: {:?}", result.err());
}

#[serial]
#[test]
fn test_devices_command_list_with_trusted_devices() {
    let env = TestEnv::setup("list_with_trusted");

    // Add some trusted devices
    {
        let mut vault = Vault::open(&env.db_path, "").unwrap();

        let trusted_devices = serde_json::json!([
            {
                "device_id": "macos-MacBookPro-a1b2c3d4",
                "first_seen": 1704067200,
                "last_seen": 1704153600,
                "sync_count": 5
            },
            {
                "device_id": "ios-iPhone15-e5f6g7h8",
                "first_seen": 1704067200,
                "last_seen": 1704153600,
                "sync_count": 3
            }
        ]);

        vault
            .set_metadata("trusted_devices", &trusted_devices.to_string())
            .unwrap();
    }

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(200));

    // List devices
    let args = DevicesArgs { remove: None };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { manage_devices(args).await });

    assert!(result.is_ok(), "List should succeed: {:?}", result.err());
}

#[serial]
#[test]
fn test_devices_command_list_with_revoked_devices() {
    let env = TestEnv::setup("list_with_revoked");

    // Add trusted and revoked devices
    {
        let mut vault = Vault::open(&env.db_path, "").unwrap();

        let trusted_devices = serde_json::json!([
            {
                "device_id": "macos-MacBookPro-a1b2c3d4",
                "first_seen": 1704067200,
                "last_seen": 1704153600,
                "sync_count": 5
            }
        ]);

        vault
            .set_metadata("trusted_devices", &trusted_devices.to_string())
            .unwrap();

        let revoked_devices = serde_json::json!([
            {
                "device_id": "ios-iPhone15-e5f6g7h8",
                "revoked_at": 1704153600
            }
        ]);

        vault
            .set_metadata("revoked_devices", &revoked_devices.to_string())
            .unwrap();
    }

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(200));

    // List devices
    let args = DevicesArgs { remove: None };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { manage_devices(args).await });

    assert!(result.is_ok(), "List should succeed: {:?}", result.err());
}

#[serial]
#[test]
fn test_devices_command_remove_device() {
    let env = TestEnv::setup("remove_device");

    // Add trusted devices - use unique IDs that won't conflict with auto-generated device ID
    {
        let mut vault = Vault::open(&env.db_path, "").unwrap();

        // First, get the current device ID so we don't use it
        let _current_device_id = vault.get_metadata("device_id").unwrap();
        let _current_device_id = _current_device_id.as_deref();

        let trusted_devices = serde_json::json!([
            {
                "device_id": "test-device-remove-001",
                "first_seen": 1704067200,
                "last_seen": 1704153600,
                "sync_count": 5
            },
            {
                "device_id": "test-device-remove-002",
                "first_seen": 1704067200,
                "last_seen": 1704153600,
                "sync_count": 3
            }
        ]);

        vault
            .set_metadata("trusted_devices", &trusted_devices.to_string())
            .unwrap();
    }

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Remove a device
    let args = DevicesArgs {
        remove: Some("test-device-remove-002".to_string()),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { manage_devices(args).await });

    assert!(result.is_ok(), "Remove should succeed: {:?}", result.err());

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Verify device was revoked
    let vault = Vault::open(&env.db_path, "").unwrap();
    let revoked_json = vault.get_metadata("revoked_devices").unwrap();
    assert!(
        revoked_json.is_some(),
        "Revoked devices metadata should exist: got {:?}",
        revoked_json
    );

    let revoked: serde_json::Value = serde_json::from_str(&revoked_json.unwrap()).unwrap();

    assert_eq!(revoked.as_array().unwrap().len(), 1);
    assert_eq!(revoked[0]["device_id"], "test-device-remove-002");
}

#[serial]
#[test]
fn test_devices_command_remove_already_revoked() {
    let env = TestEnv::setup("remove_already_revoked");

    // Add a device that's already revoked
    {
        let mut vault = Vault::open(&env.db_path, "").unwrap();

        let revoked_devices = serde_json::json!([
            {
                "device_id": "test-device-already-revoked",
                "revoked_at": 1704153600
            }
        ]);

        vault
            .set_metadata("revoked_devices", &revoked_devices.to_string())
            .unwrap();
    }

    // Give time for WAL checkpoint
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Verify the revoked device was actually saved
    {
        let vault = Vault::open(&env.db_path, "").unwrap();
        let revoked_json = vault.get_metadata("revoked_devices").unwrap();
        assert!(
            revoked_json.is_some(),
            "Revoked device should be saved before removal attempt"
        );
    }

    // Try to remove the same device again
    let args = DevicesArgs {
        remove: Some("test-device-already-revoked".to_string()),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { manage_devices(args).await });

    assert!(
        result.is_err(),
        "Should fail to remove already revoked device: {:?}",
        result
    );
}

#[serial]
#[test]
fn test_devices_command_parse_args() {
    // Test creating DevicesArgs
    let args_list = DevicesArgs { remove: None };
    assert!(args_list.remove.is_none());

    let args_remove = DevicesArgs {
        remove: Some("device-id".to_string()),
    };
    assert!(args_remove.remove.is_some());
    assert_eq!(args_remove.remove.unwrap(), "device-id");
}
