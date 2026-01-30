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

#[test]
fn test_concurrent_read_access() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Initialize database with some data
    {
        let conn = schema::initialize_database(&db_path).unwrap();
        for i in 0..10 {
            conn.execute(
                "INSERT INTO metadata (key, value) VALUES (?1, ?2)",
                (format!("key-{}", i), format!("value-{}", i)),
            )
            .unwrap();
        }
    }

    // Test concurrent reads from multiple connections
    let num_readers = 5;
    let barrier = Arc::new(Barrier::new(num_readers));
    let mut handles = vec![];

    for i in 0..num_readers {
        let barrier = Arc::clone(&barrier);
        let db_path = db_path.clone();

        let handle = thread::spawn(move || {
            // Each thread opens its own connection
            let conn = schema::initialize_database(&db_path).unwrap();

            // Wait for all threads to be ready
            barrier.wait();

            // Perform concurrent reads
            let mut stmt = conn.prepare("SELECT key, value FROM metadata").unwrap();
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?
                ))
            }).unwrap();

            let mut count = 0;
            for row in rows {
                let (key, value) = row.unwrap();
                // Verify data integrity
                assert!(key.starts_with("key-"));
                assert!(value.starts_with("value-"));
                count += 1;
            }

            // Should have read all 10 rows
            assert_eq!(count, 10, "Thread {} should read all 10 rows", i);

            count
        });

        handles.push(handle);
    }

    // Verify all threads completed successfully
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), num_readers);
    for result in results {
        assert_eq!(result, 10);
    }
}

#[test]
fn test_concurrent_read_write_access() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Initialize database
    {
        let conn = schema::initialize_database(&db_path).unwrap();
        for i in 0..5 {
            conn.execute(
                "INSERT INTO metadata (key, value) VALUES (?1, ?2)",
                (format!("key-{}", i), format!("value-{}", i)),
            )
            .unwrap();
        }
    }

    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = Arc::clone(&barrier);
    let db_path_reader = db_path.clone();
    let db_path_writer = db_path.clone();

    // Reader thread
    let reader = thread::spawn(move || {
        let conn = schema::initialize_database(&db_path_reader).unwrap();
        barrier_clone.wait();

        // Try to read - should succeed even with writer active
        // due to WAL mode allowing concurrent readers
        let mut success_count = 0;
        for _ in 0..10 {
            let mut stmt = conn.prepare("SELECT COUNT(*) FROM metadata").unwrap();
            let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
            assert!(count >= 5, "Should have at least initial rows");
            success_count += 1;
            thread::sleep(Duration::from_millis(10));
        }

        success_count
    });

    // Writer thread
    let writer = thread::spawn(move || {
        let conn = schema::initialize_database(&db_path_writer).unwrap();
        barrier.wait();

        // Write additional data
        for i in 5..15 {
            // Add small delay to allow reader to interleave
            thread::sleep(Duration::from_millis(5));
            conn.execute(
                "INSERT INTO metadata (key, value) VALUES (?1, ?2)",
                (format!("key-{}", i), format!("value-{}", i)),
            )
            .unwrap();
        }

        10 // number of writes
    });

    let reader_result = reader.join().unwrap();
    let writer_result = writer.join().unwrap();

    assert_eq!(reader_result, 10, "Reader should complete all reads");
    assert_eq!(writer_result, 10, "Writer should complete all writes");

    // Verify final state
    let conn = schema::initialize_database(&db_path).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM metadata", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 15, "Should have all 15 rows after writes");
}
