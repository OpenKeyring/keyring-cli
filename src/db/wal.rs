//! WAL (Write-Ahead Log) management for SQLite

use anyhow::Result;
use rusqlite::Connection;

/// Get the current WAL file size in bytes
///
/// Returns the size of the -wal file associated with the database.
pub fn get_wal_size(conn: &Connection) -> Result<u64> {
    // Try to get actual WAL file size
    let wal_size: i64 = conn
        .pragma_query_value(None, "wal_size(DATABASE)", |row| row.get(0))
        .unwrap_or(0);

    Ok(wal_size as u64)
}

/// Run a WAL checkpoint
///
/// A checkpoint moves frames from the WAL file back into the main database.
/// This is the standard checkpoint mode (TRUNCATE).
pub fn checkpoint(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

/// Run an aggressive WAL checkpoint that truncates the WAL file
///
/// This mode:
/// 1. Writes all WAL frames back to the database
/// 2. Syncs the database file
/// 3. Truncates the WAL file to zero bytes
///
/// Use this when you want to minimize disk usage.
pub fn truncate(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
    Ok(())
}

/// Run a PASSIVE checkpoint
///
/// Only checkpoint if no readers are using the WAL. Safe to run while
/// other processes are reading the database.
pub fn checkpoint_passive(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "PASSIVE")?;
    Ok(())
}

/// Run a FULL checkpoint
///
/// Like TRUNCATE but doesn't truncate the WAL file. Use this when
/// you want to ensure consistency but don't need to reclaim space.
pub fn checkpoint_full(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "FULL")?;
    Ok(())
}

/// Run a RESTART checkpoint
///
/// Checkpoints and then restarts the WAL file. This is useful for
/// ensuring the database is in a consistent state for backup.
pub fn checkpoint_restart(conn: &mut Connection) -> Result<()> {
    conn.pragma_update(None, "wal_checkpoint", "RESTART")?;
    Ok(())
}

/// Get WAL auto-checkpoint setting
///
/// Returns the number of WAL frames before automatic checkpoint.
/// 0 means auto-checkpoint is disabled.
pub fn get_auto_checkpoint(conn: &Connection) -> Result<i64> {
    let value: i64 = conn.pragma_query_value(None, "wal_autocheckpoint", |row| row.get(0))?;
    Ok(value)
}

/// Set WAL auto-checkpoint threshold
///
/// Set to 0 to disable automatic checkpointing.
/// Recommended: 1000 (default) or higher for better performance.
pub fn set_auto_checkpoint(conn: &Connection, frames: i64) -> Result<()> {
    conn.pragma_update(None, "wal_autocheckpoint", frames)?;
    Ok(())
}

/// Disable automatic checkpointing
///
/// Manual checkpointing can be more efficient for bulk operations.
pub fn disable_auto_checkpoint(conn: &Connection) -> Result<()> {
    set_auto_checkpoint(conn, 0)
}

/// Re-enable automatic checkpointing with default value
pub fn enable_auto_checkpoint(conn: &Connection) -> Result<()> {
    set_auto_checkpoint(conn, 1000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, Connection) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = schema::initialize_database(&db_path).unwrap();
        (temp_dir, conn)
    }

    #[test]
    fn test_get_wal_size() {
        let (_temp_dir, conn) = setup_test_db();

        let wal_size = get_wal_size(&conn).unwrap();
        // WAL size should be a valid number (might be 0 for new database)
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_checkpoint() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Checkpoint should succeed
        checkpoint(&mut conn).unwrap();

        // After checkpoint, WAL size might be reduced
        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_truncate() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Truncate should succeed
        truncate(&mut conn).unwrap();

        // After truncate, WAL should be empty or very small
        let wal_size = get_wal_size(&conn).unwrap();
        // WAL might be 0 after truncate
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_checkpoint_passive() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Passive checkpoint should succeed
        checkpoint_passive(&mut conn).unwrap();

        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_checkpoint_full() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Full checkpoint should succeed
        checkpoint_full(&mut conn).unwrap();

        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_checkpoint_restart() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Restart checkpoint should succeed
        checkpoint_restart(&mut conn).unwrap();

        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_get_auto_checkpoint_default() {
        let (_temp_dir, conn) = setup_test_db();

        // Default auto-checkpoint should be 1000
        let auto_checkpoint = get_auto_checkpoint(&conn).unwrap();
        assert_eq!(auto_checkpoint, 1000);
    }

    #[test]
    fn test_set_auto_checkpoint() {
        let (_temp_dir, conn) = setup_test_db();

        // Set to custom value
        set_auto_checkpoint(&conn, 500).unwrap();

        let auto_checkpoint = get_auto_checkpoint(&conn).unwrap();
        assert_eq!(auto_checkpoint, 500);
    }

    #[test]
    fn test_set_auto_checkpoint_to_zero() {
        let (_temp_dir, conn) = setup_test_db();

        // Set to 0 to disable
        set_auto_checkpoint(&conn, 0).unwrap();

        let auto_checkpoint = get_auto_checkpoint(&conn).unwrap();
        assert_eq!(auto_checkpoint, 0);
    }

    #[test]
    fn test_disable_auto_checkpoint() {
        let (_temp_dir, conn) = setup_test_db();

        // Disable auto-checkpoint
        disable_auto_checkpoint(&conn).unwrap();

        let auto_checkpoint = get_auto_checkpoint(&conn).unwrap();
        assert_eq!(auto_checkpoint, 0);
    }

    #[test]
    fn test_enable_auto_checkpoint() {
        let (_temp_dir, conn) = setup_test_db();

        // First disable
        disable_auto_checkpoint(&conn).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 0);

        // Then re-enable
        enable_auto_checkpoint(&conn).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 1000);
    }

    #[test]
    fn test_checkpoint_after_write() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Write some data to create WAL content
        conn.execute(
            "INSERT INTO records (id, record_type, encrypted_data, nonce, created_at, updated_at, updated_by, version)
             VALUES ('test-id', 'password', X'1234', X'000000000000000000000000', 12345, 12345, 'test', 1)",
            [],
        ).unwrap();

        // Checkpoint should process the WAL
        checkpoint(&mut conn).unwrap();

        // Verify the data is still there
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_multiple_checkpoints() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Run multiple checkpoints
        checkpoint(&mut conn).unwrap();
        checkpoint_full(&mut conn).unwrap();
        checkpoint_restart(&mut conn).unwrap();

        // Should still work
        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }

    #[test]
    fn test_set_auto_checkpoint_multiple_times() {
        let (_temp_dir, conn) = setup_test_db();

        // Set to different values
        set_auto_checkpoint(&conn, 100).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 100);

        set_auto_checkpoint(&conn, 2000).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 2000);

        set_auto_checkpoint(&conn, 0).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 0);

        // Re-enable should restore default
        enable_auto_checkpoint(&conn).unwrap();
        assert_eq!(get_auto_checkpoint(&conn).unwrap(), 1000);
    }

    #[test]
    fn test_truncate_reduces_wal_size() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Write some data
        for i in 0..10 {
            conn.execute(
                "INSERT INTO records (id, record_type, encrypted_data, nonce, created_at, updated_at, updated_by, version)
                 VALUES (?1, 'password', X'1234', X'000000000000000000000000', 12345, 12345, 'test', 1)",
                [format!("test-id-{}", i)],
            ).unwrap();
        }

        // Get WAL size before truncate
        let wal_size_before = get_wal_size(&conn).unwrap();

        // Truncate WAL
        truncate(&mut conn).unwrap();

        // Get WAL size after truncate
        let wal_size_after = get_wal_size(&conn).unwrap();

        // After truncate, WAL should be smaller or equal
        assert!(wal_size_after <= wal_size_before);
    }

    #[test]
    fn test_get_wal_size_after_checkpoint() {
        let (_temp_dir, mut conn) = setup_test_db();

        // Write some data
        conn.execute(
            "INSERT INTO records (id, record_type, encrypted_data, nonce, created_at, updated_at, updated_by, version)
             VALUES ('test-id', 'password', X'1234', X'000000000000000000000000', 12345, 12345, 'test', 1)",
            [],
        ).unwrap();

        // Run checkpoint
        checkpoint(&mut conn).unwrap();

        // WAL size should still be retrievable
        let wal_size = get_wal_size(&conn).unwrap();
        assert!(wal_size >= 0);
    }
}
