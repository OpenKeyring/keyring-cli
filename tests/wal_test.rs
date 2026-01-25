use keyring_cli::db::schema;
use keyring_cli::db::wal;
use tempfile::TempDir;

#[test]
fn test_wal_checkpoint() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut conn = schema::initialize_database(&db_path).unwrap();

    // Create some WAL activity
    for i in 0..100 {
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?1, ?2)",
            (format!("key-{}", i), format!("value-{}", i)),
        )
        .unwrap();
    }

    // Get WAL size before checkpoint
    let wal_size_before = wal::get_wal_size(&conn).unwrap();

    // Run checkpoint
    wal::checkpoint(&mut conn).unwrap();

    // Get WAL size after checkpoint
    let wal_size_after = wal::get_wal_size(&conn).unwrap();

    // WAL should be smaller or equal after checkpoint
    assert!(wal_size_after <= wal_size_before);
}

#[test]
fn test_wal_truncate() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut conn = schema::initialize_database(&db_path).unwrap();

    // Create some WAL activity
    for i in 0..100 {
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?1, ?2)",
            (format!("key-{}", i), format!("value-{}", i)),
        )
        .unwrap();
    }

    // Run truncate checkpoint (most aggressive)
    wal::truncate(&mut conn).unwrap();

    // WAL file should be minimal
    let wal_size = wal::get_wal_size(&conn).unwrap();
    assert!(wal_size < 4096, "WAL should be minimal after truncate");
}
