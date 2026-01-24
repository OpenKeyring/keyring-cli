use keyring_cli::db::{models, schema};
use tempfile::TempDir;

#[test]
fn test_schema_init() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = schema::initialize_database(&db_path).unwrap();

    // Check tables exist
    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row: &rusqlite::Row| row.get(0),
        )
        .unwrap();
    assert!(table_count >= 5); // records, tags, record_tags, metadata, sync_state
}

#[test]
fn test_record_model() {
    let record = models::Record {
        id: "test-id".to_string(),
        record_type: models::RecordType::Password,
        encrypted_data: vec![1, 2, 3],
        nonce: vec![4, 5, 6],
        created_at: 12345,
        updated_at: 12345,
        updated_by: "test-device".to_string(),
        version: 1,
        deleted: false,
    };
    assert_eq!(record.id, "test-id");
}
