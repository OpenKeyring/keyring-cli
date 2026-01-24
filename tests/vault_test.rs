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

    // Verify record was inserted
    let count: i64 = vault.conn.query_row(
        "SELECT COUNT(*) FROM records WHERE id = ?1",
        &[&record.id.to_string()],
        |row: &rusqlite::Row| row.get(0),
    ).unwrap();
    assert_eq!(count, 1, "Record should be inserted into database");

    // Verify tags were inserted
    let tag_count: i64 = vault.conn.query_row(
        "SELECT COUNT(*) FROM record_tags WHERE record_id = ?1",
        &[&record.id.to_string()],
        |row: &rusqlite::Row| row.get(0),
    ).unwrap();
    assert_eq!(tag_count, 2, "Both tags should be linked to record");
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

    // Verify both records exist
    let count: i64 = vault.conn.query_row(
        "SELECT COUNT(*) FROM records",
        [],
        |row: &rusqlite::Row| row.get(0),
    ).unwrap();
    assert_eq!(count, 2, "Both records should be inserted");

    // Verify tags are shared (work tag should be used by both records)
    let unique_tags: i64 = vault.conn.query_row(
        "SELECT COUNT(*) FROM tags",
        [],
        |row: &rusqlite::Row| row.get(0),
    ).unwrap();
    assert_eq!(unique_tags, 3, "Should have 3 unique tags: work, important, personal");
}

#[test]
fn test_add_record_with_duplicate_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Record with duplicate tag names (should be deduplicated)
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

    // Verify deduplication: only 2 unique tags should be linked
    let tag_count: i64 = vault.conn.query_row(
        "SELECT COUNT(*) FROM record_tags WHERE record_id = ?1",
        &[&record.id.to_string()],
        |row: &rusqlite::Row| row.get(0),
    ).unwrap();
    assert_eq!(tag_count, 2, "Duplicate tags should be deduplicated to 2 unique tags");
}

#[test]
fn test_get_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "encrypted-data".to_string(),
        name: "test-record".to_string(),
        username: Some("user@example.com".to_string()),
        url: None,
        notes: None,
        tags: vec!["work".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    let retrieved = vault.get_record(&record.id.to_string()).unwrap();
    assert_eq!(retrieved.id, record.id);
    // Note: name is stored encrypted, will be decoded when crypto module is integrated
    assert!(retrieved.name.is_empty());
    assert_eq!(retrieved.tags.len(), 1);
    assert_eq!(retrieved.tags[0], "work");
}

#[test]
fn test_list_records() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record1 = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "data1".to_string(),
        name: "record1".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let record2 = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::SshKey,
        encrypted_data: "data2".to_string(),
        name: "record2".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record1).unwrap();
    vault.add_record(&record2).unwrap();

    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 2);
}

#[test]
fn test_list_records_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record1 = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "data1".to_string(),
        name: "record1".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec!["work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record1).unwrap();

    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].tags.len(), 2);
    assert!(records[0].tags.contains(&"work".to_string()));
    assert!(records[0].tags.contains(&"important".to_string()));
}

#[test]
fn test_list_records_empty() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let vault = Vault::open(&db_path, "test-password").unwrap();

    // List records when database is empty
    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 0);
    assert!(records.is_empty());
}

#[test]
fn test_update_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let mut record = Record {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: "original-data".to_string(),
        name: "original-name".to_string(),
        username: None,
        url: None,
        notes: None,
        tags: vec!["tag1".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    record.encrypted_data = "updated-data".to_string();
    record.tags = vec!["tag2".to_string()];

    assert!(vault.update_record(&record).is_ok());

    let retrieved = vault.get_record(&record.id.to_string()).unwrap();
    assert_eq!(retrieved.encrypted_data, "updated-data");
    assert_eq!(retrieved.tags.len(), 1);
    assert_eq!(retrieved.tags[0], "tag2");
}
