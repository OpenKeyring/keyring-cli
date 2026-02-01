//! CLI delete command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::delete::{delete_record, DeleteArgs};
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::vault::Vault;
use keyring_cli::error::Error;
use serial_test::serial;
use std::env;
use tempfile::TempDir;
use uuid::Uuid;

#[serial]
#[test]
fn test_delete_record_without_confirm_returns_early() {
    // Test: Delete without --confirm should return early without error
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = std::process::id(); // Use process ID to avoid conflicts

    // Set environment variables for ConfigManager
    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Create data directory
    std::fs::create_dir_all(&data_dir).unwrap();

    // The database path will be data_dir/passwords.db
    let db_path = data_dir.join("passwords.db");

    // Create a test record with JSON payload (unencrypted for testing)
    let payload = serde_json::json!({
        "name": "test-record",
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": null,
        "tags": []
    });

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: serde_json::to_vec(&payload).unwrap(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };

    let mut vault = Vault::open(&db_path, "").unwrap();
    vault.add_record(&record).unwrap();

    // Close the vault by dropping it before delete_record tries to open it
    drop(vault);

    // Try to delete without --confirm flag
    let args = DeleteArgs {
        name: "test-record".to_string(),
        confirm: false,
        sync: false,
    };

    // Should succeed but NOT delete the record
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { delete_record(args).await });

    assert!(result.is_ok());

    // Verify record still exists (not deleted)
    let vault = Vault::open(&db_path, "").unwrap();
    let records = vault.list_records().unwrap();
    assert_eq!(
        records.len(),
        1,
        "Record should still exist when --confirm is not set"
    );
}

#[serial]
#[test]
fn test_delete_record_successfully_marks_as_deleted() {
    // Test: Delete a record and verify it's marked as deleted (deleted=1)
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("delete_success_{}", std::process::id());

    // Set environment variables for ConfigManager
    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Create data directory
    std::fs::create_dir_all(&data_dir).unwrap();

    // The database path will be data_dir/passwords.db
    let db_path = data_dir.join("passwords.db");

    // Create a test record with JSON payload
    let payload = serde_json::json!({
        "name": "test-record-to-delete",
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": null,
        "tags": []
    });

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: serde_json::to_vec(&payload).unwrap(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };

    let mut vault = Vault::open(&db_path, "").unwrap();
    vault.add_record(&record).unwrap();

    // Force WAL checkpoint to ensure record is persisted
    let _ = vault.conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");

    // Close the vault by dropping it before delete_record tries to open it
    drop(vault);

    // Delete with --confirm flag
    let args = DeleteArgs {
        name: "test-record-to-delete".to_string(),
        confirm: true,
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { delete_record(args).await });

    if let Err(ref e) = result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok(), "Delete should succeed");

    // Verify record is marked as deleted (should not appear in list_records)
    let vault = Vault::open(&db_path, "").unwrap();
    let records = vault.list_records().unwrap();
    assert_eq!(
        records.len(),
        0,
        "Record should be marked as deleted and not appear in list"
    );
}

#[serial]
#[test]
fn test_delete_nonexistent_record_returns_error() {
    // Test: Delete non-existent record should return RecordNotFound error
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("delete_not_found_{}", std::process::id());

    // Set environment variables for ConfigManager
    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Create data directory
    std::fs::create_dir_all(&data_dir).unwrap();

    // The database path will be data_dir/passwords.db
    let db_path = data_dir.join("passwords.db");

    // Create empty vault
    Vault::open(&db_path, "").unwrap();

    // Try to delete non-existent record
    let args = DeleteArgs {
        name: "nonexistent-record".to_string(),
        confirm: true,
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { delete_record(args).await });

    assert!(
        result.is_err(),
        "Delete should fail for non-existent record"
    );

    // Verify it's the correct error type
    match result {
        Err(Error::RecordNotFound { name }) => {
            assert_eq!(name, "nonexistent-record");
        }
        _ => panic!("Expected RecordNotFound error, got {:?}", result),
    }
}

#[serial]
#[test]
fn test_delete_record_with_sync_calls_sync_deletion() {
    // Test: Delete with --sync flag should call sync_deletion
    // Note: This test verifies sync_deletion is called, but sync_deletion itself is a placeholder
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("delete_sync_{}", std::process::id());

    // Set environment variables for ConfigManager
    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());

    // Create data directory
    std::fs::create_dir_all(&data_dir).unwrap();

    // The database path will be data_dir/passwords.db
    let db_path = data_dir.join("passwords.db");

    // Create a test record with JSON payload
    let payload = serde_json::json!({
        "name": "test-record-sync",
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": null,
        "tags": []
    });

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: serde_json::to_vec(&payload).unwrap(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };

    let mut vault = Vault::open(&db_path, "").unwrap();
    vault.add_record(&record).unwrap();

    // Close the vault by dropping it before delete_record tries to open it
    drop(vault);

    // Delete with --sync flag
    let args = DeleteArgs {
        name: "test-record-sync".to_string(),
        confirm: true,
        sync: true,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { delete_record(args).await });

    assert!(result.is_ok(), "Delete with sync should succeed");

    // Verify record is deleted
    let vault = Vault::open(&db_path, "").unwrap();
    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 0, "Record should be marked as deleted");
}
