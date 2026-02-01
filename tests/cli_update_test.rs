//! CLI update command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::update::{update_record, UpdateArgs};
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::vault::Vault;
use keyring_cli::error::Error;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_update_username_field() {
    // Test: Update the username field of a record
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_username_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");

    // Create initial record
    let payload = serde_json::json!({
        "name": "test-record",
        "username": "old@example.com",
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
    drop(vault);

    // Update username
    let args = UpdateArgs {
        name: "test-record".to_string(),
        password: None,
        username: Some("new@example.com".to_string()),
        url: None,
        notes: None,
        tags: vec![],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(result.is_ok(), "Update should succeed");

    // Verify username was updated
    let vault = Vault::open(&db_path, "").unwrap();
    let updated = vault.find_record_by_name("test-record").unwrap().unwrap();
    let updated_payload: serde_json::Value =
        serde_json::from_slice(&updated.encrypted_data).unwrap();
    assert_eq!(updated_payload["username"], "new@example.com");
}

#[test]
fn test_update_url_field() {
    // Test: Update the URL field of a record
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_url_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");

    // Create initial record
    let payload = serde_json::json!({
        "name": "test-record-url",
        "username": "user@example.com",
        "password": "password123",
        "url": "https://old.example.com",
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
    drop(vault);

    // Update URL
    let args = UpdateArgs {
        name: "test-record-url".to_string(),
        password: None,
        username: None,
        url: Some("https://new.example.com".to_string()),
        notes: None,
        tags: vec![],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(result.is_ok(), "Update should succeed");

    // Verify URL was updated
    let vault = Vault::open(&db_path, "").unwrap();
    let updated = vault
        .find_record_by_name("test-record-url")
        .unwrap()
        .unwrap();
    let updated_payload: serde_json::Value =
        serde_json::from_slice(&updated.encrypted_data).unwrap();
    assert_eq!(updated_payload["url"], "https://new.example.com");
}

#[test]
fn test_update_notes_field() {
    // Test: Update the notes field of a record
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_notes_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");

    // Create initial record
    let payload = serde_json::json!({
        "name": "test-record-notes",
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": "Old notes",
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
    drop(vault);

    // Update notes
    let args = UpdateArgs {
        name: "test-record-notes".to_string(),
        password: None,
        username: None,
        url: None,
        notes: Some("New updated notes".to_string()),
        tags: vec![],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(result.is_ok(), "Update should succeed");

    // Verify notes were updated
    let vault = Vault::open(&db_path, "").unwrap();
    let updated = vault
        .find_record_by_name("test-record-notes")
        .unwrap()
        .unwrap();
    let updated_payload: serde_json::Value =
        serde_json::from_slice(&updated.encrypted_data).unwrap();
    assert_eq!(updated_payload["notes"], "New updated notes");
}

#[test]
fn test_update_tags_replace() {
    // Test: Update tags (should replace existing tags)
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_tags_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");

    // Create initial record with existing tags in the database
    let payload = serde_json::json!({
        "name": "test-record-tags",
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": null,
        "tags": ["old-tag"]
    });

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: serde_json::to_vec(&payload).unwrap(),
        nonce: [0u8; 12],
        tags: vec!["old-tag".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };

    let mut vault = Vault::open(&db_path, "").unwrap();
    vault.add_record(&record).unwrap();
    drop(vault);

    // Update tags
    let args = UpdateArgs {
        name: "test-record-tags".to_string(),
        password: None,
        username: None,
        url: None,
        notes: None,
        tags: vec!["new-tag".to_string(), "another-tag".to_string()],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(result.is_ok(), "Update should succeed");

    // Verify tags were replaced (check both encrypted data and database tags)
    let vault = Vault::open(&db_path, "").unwrap();
    let updated = vault
        .find_record_by_name("test-record-tags")
        .unwrap()
        .unwrap();
    let updated_payload: serde_json::Value =
        serde_json::from_slice(&updated.encrypted_data).unwrap();
    let updated_tags: Vec<String> = updated_payload["tags"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .map(String::from)
        .collect();

    // Sort for comparison since order may vary
    let mut expected_tags = vec!["new-tag", "another-tag"];
    expected_tags.sort();
    let mut sorted_updated_tags = updated_tags.clone();
    sorted_updated_tags.sort();

    assert_eq!(sorted_updated_tags, expected_tags);

    let mut sorted_db_tags = updated.tags.clone();
    sorted_db_tags.sort();
    assert_eq!(sorted_db_tags, expected_tags);
}

#[test]
fn test_update_nonexistent_record_returns_error() {
    // Test: Update non-existent record should return RecordNotFound error
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_not_found_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    Vault::open(&db_path, "").unwrap();

    // Try to update non-existent record
    let args = UpdateArgs {
        name: "nonexistent-record".to_string(),
        password: None,
        username: Some("test@example.com".to_string()),
        url: None,
        notes: None,
        tags: vec![],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(
        result.is_err(),
        "Update should fail for non-existent record"
    );

    // Verify it's the correct error type
    match result {
        Err(Error::RecordNotFound { name }) => {
            assert_eq!(name, "nonexistent-record");
        }
        _ => panic!("Expected RecordNotFound error, got {:?}", result),
    }
}

#[test]
fn test_update_password_with_encryption() {
    // Test: Update password field with encryption
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("update_password_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    // Set master password for encryption
    std::env::set_var("OK_MASTER_PASSWORD", "test-master-password");

    let db_path = data_dir.join("passwords.db");

    // Create initial record
    let payload = serde_json::json!({
        "name": "test-record-password",
        "username": "user@example.com",
        "password": "old-password",
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
    drop(vault);

    // Update password
    let args = UpdateArgs {
        name: "test-record-password".to_string(),
        password: Some("new-password-456".to_string()),
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        sync: false,
    };

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { update_record(args).await });

    assert!(result.is_ok(), "Password update should succeed");

    // Verify password was updated (encrypted data changed)
    let vault = Vault::open(&db_path, "").unwrap();
    let updated = vault
        .find_record_by_name("test-record-password")
        .unwrap()
        .unwrap();
    let updated_payload: serde_json::Value =
        serde_json::from_slice(&updated.encrypted_data).unwrap();
    assert_eq!(updated_payload["password"], "new-password-456");
}
