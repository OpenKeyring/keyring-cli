use keyring_cli::db::vault::Vault;
use keyring_cli::db::models::{Record, RecordType};
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_add_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "encrypted-data".to_string(),
        name: "test-record".to_string(),
        username: Some("user@example.com".to_string()),
        url: Some("https://example.com".to_string()),
        notes: Some("Test notes".to_string()),
        tags: vec!["work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(vault.add_record(&record).is_ok());
}

#[test]
fn test_add_record_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Add first record with tags
    let record1 = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "encrypted-data-1".to_string(),
        name: "record-1".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec!["work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(vault.add_record(&record1).is_ok());

    // Add second record with overlapping tags
    let record2 = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::SshKey,
        encrypted_data: "encrypted-data-2".to_string(),
        name: "record-2".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec!["work".to_string(), "personal".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(vault.add_record(&record2).is_ok());
}

#[test]
fn test_add_record_with_duplicate_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Record with duplicate tag names (should be deduplicated by database)
    let record = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "encrypted-data".to_string(),
        name: "test-record".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec!["work".to_string(), "work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Should not fail even with duplicate tag names
    assert!(vault.add_record(&record).is_ok());
}
