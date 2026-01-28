use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::db::vault::Vault;
use tempfile::TempDir;
use uuid::Uuid;

#[test]
fn test_add_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    assert!(vault.add_record(&record).is_ok());

    // Verify record was inserted
    let count: i64 = vault
        .conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE id = ?1",
            &[&record.id.to_string()],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "Record should be inserted into database");

    // Verify tags were inserted
    let tag_count: i64 = vault
        .conn
        .query_row(
            "SELECT COUNT(*) FROM record_tags WHERE record_id = ?1",
            &[&record.id.to_string()],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap();
    assert_eq!(tag_count, 2, "Both tags should be linked to record");
}

#[test]
fn test_add_record_with_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Add first record with tags
    let record1 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data-1".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["work".to_string(), "important".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(vault.add_record(&record1).is_ok());

    // Add second record with overlapping tags
    let record2 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::SshKey,
        encrypted_data: b"encrypted-data-2".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["work".to_string(), "personal".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert!(vault.add_record(&record2).is_ok());

    // Verify both records exist
    let count: i64 = vault
        .conn
        .query_row("SELECT COUNT(*) FROM records", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 2, "Both records should be inserted");

    // Verify tags are shared (work tag should be used by both records)
    let unique_tags: i64 = vault
        .conn
        .query_row("SELECT COUNT(*) FROM tags", [], |row: &rusqlite::Row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        unique_tags, 3,
        "Should have 3 unique tags: work, important, personal"
    );
}

#[test]
fn test_add_record_with_duplicate_tags() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Record with duplicate tag names (should be deduplicated)
    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [0u8; 12],
        tags: vec![
            "work".to_string(),
            "work".to_string(),
            "important".to_string(),
        ],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Should not fail even with duplicate tag names
    assert!(vault.add_record(&record).is_ok());

    // Verify deduplication: only 2 unique tags should be linked
    let tag_count: i64 = vault
        .conn
        .query_row(
            "SELECT COUNT(*) FROM record_tags WHERE record_id = ?1",
            &[&record.id.to_string()],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        tag_count, 2,
        "Duplicate tags should be deduplicated to 2 unique tags"
    );
}

#[test]
fn test_get_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["work".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    let retrieved = vault.get_record(&record.id.to_string()).unwrap();
    assert_eq!(retrieved.id, record.id);
    assert_eq!(retrieved.tags.len(), 1);
    assert_eq!(retrieved.tags[0], "work");
}

#[test]
fn test_list_records() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let record1 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data1".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let record2 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::SshKey,
        encrypted_data: b"data2".to_vec(),
        nonce: [0u8; 12],
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

    let record1 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data1".to_vec(),
        nonce: [0u8; 12],
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

    let mut record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"original-data".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["tag1".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    // Get initial version (should be 1 after add_record)
    let initial_version: i64 = vault
        .conn
        .query_row(
            "SELECT version FROM records WHERE id = ?1",
            &[&record.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(initial_version, 1, "Initial version should be 1");

    record.encrypted_data = b"updated-data".to_vec();
    record.tags = vec!["tag2".to_string()];

    assert!(vault.update_record(&record).is_ok());

    let retrieved = vault.get_record(&record.id.to_string()).unwrap();
    assert_eq!(retrieved.encrypted_data, b"updated-data".to_vec());
    assert_eq!(retrieved.tags.len(), 1);
    assert_eq!(retrieved.tags[0], "tag2");

    // Verify version was incremented
    let updated_version: i64 = vault
        .conn
        .query_row(
            "SELECT version FROM records WHERE id = ?1",
            &[&record.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        updated_version, 2,
        "Version should be incremented after update"
    );
}

#[test]
fn test_soft_delete_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Create a record
    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [0u8; 12],
        tags: vec!["test-tag".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    // Verify record exists in list
    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 1);

    // Soft delete the record
    vault.delete_record(&record.id.to_string()).unwrap();

    // Record should not appear in list anymore
    let records = vault.list_records().unwrap();
    assert_eq!(records.len(), 0);

    // Record should still exist in database with deleted=1
    let count: i64 = vault
        .conn
        .query_row(
            "SELECT COUNT(*) FROM records WHERE id = ?1 AND deleted = 1",
            [&record.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // Sync state should be updated
    let sync_state = vault.get_sync_state(&record.id.to_string()).unwrap();
    assert!(sync_state.is_some());
}

#[test]
fn test_delete_nonexistent_record() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    let result = vault.delete_record(&Uuid::new_v4().to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_find_record_by_name_not_found() {
    // Test: Finding a non-existent record should return None
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let vault = Vault::open(&db_path, "test-password").unwrap();

    // Try to find a record that doesn't exist
    let result = vault.find_record_by_name("nonexistent-record");
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_none(),
        "Should return None for non-existent record"
    );
}

#[test]
fn test_find_record_by_name_success() {
    // Test: Find an existing record by its decrypted name
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Create a record with a specific name in the encrypted payload
    let record_name = "my-test-record";
    let payload = serde_json::json!({
        "name": record_name,
        "username": "user@example.com",
        "password": "password123",
        "url": null,
        "notes": null,
        "tags": []
    });

    // Encrypt the payload (use simple encryption for testing)
    let encrypted_data = serde_json::to_vec(&payload).unwrap();
    let nonce = [0u8; 12];

    let record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags: vec!["test-tag".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record).unwrap();

    // Find the record by name
    let result = vault.find_record_by_name(record_name);
    assert!(result.is_ok());
    let found_record = result.unwrap();
    assert!(found_record.is_some(), "Should find the existing record");

    let found = found_record.unwrap();
    assert_eq!(found.id, record.id, "Should return the correct record");
    assert_eq!(found.tags.len(), 1, "Should include tags");
    assert_eq!(found.tags[0], "test-tag");
}

#[test]
fn test_get_sync_stats_empty_database() {
    // Test: Get sync stats from empty database returns zeros
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let vault = Vault::open(&db_path, "test-password").unwrap();

    let stats = vault.get_sync_stats().unwrap();

    assert_eq!(stats.total, 0, "Total records should be 0");
    assert_eq!(stats.pending, 0, "Pending records should be 0");
    assert_eq!(stats.synced, 0, "Synced records should be 0");
    assert_eq!(stats.conflicts, 0, "Conflicts should be 0");
}

#[test]
fn test_get_sync_stats_with_records() {
    // Test: Get sync stats counts total, pending, synced records correctly
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Create 3 records
    let record1 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data1".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let record2 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data2".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let record3 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data3".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record1).unwrap();
    vault.add_record(&record2).unwrap();
    vault.add_record(&record3).unwrap();

    // Manually set sync states: 1 pending, 1 synced, 1 conflict
    // SyncStatus values: 0 = Pending, 1 = Synced, 2 = Conflict

    let _ = vault.conn.execute(
        "INSERT OR REPLACE INTO sync_state (record_id, sync_status) VALUES (?1, ?2)",
        (&record1.id.to_string(), 0i32), // Pending
    );
    let _ = vault.conn.execute(
        "INSERT OR REPLACE INTO sync_state (record_id, sync_status) VALUES (?1, ?2)",
        (&record2.id.to_string(), 1i32), // Synced
    );
    let _ = vault.conn.execute(
        "INSERT OR REPLACE INTO sync_state (record_id, sync_status) VALUES (?1, ?2)",
        (&record3.id.to_string(), 2i32), // Conflict
    );

    let stats = vault.get_sync_stats().unwrap();

    assert_eq!(stats.total, 3, "Total records should be 3");
    assert_eq!(stats.pending, 1, "Pending records should be 1");
    assert_eq!(stats.synced, 1, "Synced records should be 1");
    assert_eq!(stats.conflicts, 1, "Conflicts should be 1");
}

#[test]
fn test_get_pending_records_empty() {
    // Test: Get pending records from empty database returns empty vec
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let vault = Vault::open(&db_path, "test-password").unwrap();

    let pending = vault.get_pending_records().unwrap();
    assert_eq!(pending.len(), 0, "Should return empty vec when no records");
}

#[test]
fn test_get_pending_records_with_pending() {
    // Test: Get pending records returns records with sync_status = Pending
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut vault = Vault::open(&db_path, "test-password").unwrap();

    // Create 2 records
    let record1 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data1".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let record2 = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"data2".to_vec(),
        nonce: [0u8; 12],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    vault.add_record(&record1).unwrap();
    vault.add_record(&record2).unwrap();

    // Mark record2 as synced (record1 is already pending from add_record)
    let _ = vault.conn.execute(
        "UPDATE sync_state SET sync_status = ?1 WHERE record_id = ?2",
        (1i32, record2.id.to_string()), // Synced
    );

    let pending = vault.get_pending_records().unwrap();
    assert_eq!(pending.len(), 1, "Should return 1 pending record");
    assert_eq!(
        pending[0].id, record1.id,
        "Should return record1 as pending"
    );
}
