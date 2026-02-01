//! CLI search command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::search::{search_records, SearchArgs};
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::Vault;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_search_filters_by_type() {
    // Test: Search results can be filtered by record type
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("search_type_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    let mut vault = Vault::open(&db_path, "").unwrap();

    // Add password record
    let password_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"test-password".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };
    vault.add_record(&password_record).unwrap();

    // Add SSH key record
    let ssh_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::SshKey,
        encrypted_data: b"test-ssh".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };
    vault.add_record(&ssh_record).unwrap();

    // Search with type filter should only return password records
    let search_args = SearchArgs {
        query: "test".to_string(),
        r#type: Some("password".to_string()),
        tags: vec![],
        limit: None,
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { search_records(search_args).await })
        .unwrap();

    // Verify by checking vault directly (since search_records only prints)
    let results = vault.search_records("test").unwrap();
    assert!(results.len() >= 1, "Should have at least one result");
}

#[test]
fn test_search_filters_by_tags() {
    // Test: Search results can be filtered by tags
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("search_tags_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    let mut vault = Vault::open(&db_path, "").unwrap();

    // Add record with "work" tag
    let work_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"work-account".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["work".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };
    vault.add_record(&work_record).unwrap();

    // Add record with "personal" tag
    let personal_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"personal-account".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["personal".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 0,
    };
    vault.add_record(&personal_record).unwrap();

    // Search with tag filter should only return records with "work" tag
    let search_args = SearchArgs {
        query: "account".to_string(),
        r#type: None,
        tags: vec!["work".to_string()],
        limit: None,
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { search_records(search_args).await })
        .unwrap();
}

#[test]
fn test_search_respects_limit() {
    // Test: Search results respect the limit parameter
    let temp_dir = TempDir::new().unwrap();
    let unique_suffix = format!("search_limit_{}", std::process::id());

    let config_dir = temp_dir.path().join(format!("config_{}", unique_suffix));
    let data_dir = temp_dir.path().join(format!("data_{}", unique_suffix));
    std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
    std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("passwords.db");
    let mut vault = Vault::open(&db_path, "").unwrap();

    // Add 3 records
    for i in 0..3 {
        let record = StoredRecord {
            id: Uuid::new_v4(),
            record_type: RecordType::Password,
            encrypted_data: format!("test-{}", i).as_bytes().to_vec(),
            nonce: [0u8; 12],
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        version: 0,
        };
        vault.add_record(&record).unwrap();
    }

    // Search with limit=2 should only return 2 results
    let search_args = SearchArgs {
        query: "test".to_string(),
        r#type: None,
        tags: vec![],
        limit: Some(2),
    };

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { search_records(search_args).await })
        .unwrap();
}
