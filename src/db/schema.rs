use anyhow::Result;
use rusqlite::Connection;

/// Initialize database schema with WAL mode
///
/// Creates all tables and enables Write-Ahead Logging for concurrent access.
pub fn initialize_database(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    // Create tables
    create_records_table(&conn)?;
    create_tags_table(&conn)?;
    create_metadata_table(&conn)?;
    create_sync_state_table(&conn)?;
    create_mcp_sessions_table(&conn)?;
    create_mcp_policies_table(&conn)?;

    Ok(conn)
}

fn create_records_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            record_type TEXT NOT NULL,
            encrypted_data BLOB NOT NULL,
            nonce BLOB NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            updated_by TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 1,
            deleted INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Create indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_records_type ON records(record_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_records_updated ON records(updated_at DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_records_deleted ON records(deleted)",
        [],
    )?;

    Ok(())
}

fn create_tags_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS record_tags (
            record_id TEXT NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (record_id, tag_id),
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

fn create_metadata_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn create_sync_state_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_state (
            record_id TEXT PRIMARY KEY,
            cloud_updated_at INTEGER,
            sync_status INTEGER NOT NULL,
            FOREIGN KEY (record_id) REFERENCES records(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

fn create_mcp_sessions_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mcp_sessions (
            id TEXT PRIMARY KEY,
            approved_credentials TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_activity INTEGER NOT NULL,
            ttl_seconds INTEGER NOT NULL
        )",
        [],
    )?;

    // Create index for efficient session lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mcp_sessions_last_activity
         ON mcp_sessions(last_activity DESC)",
        [],
    )?;

    Ok(())
}

fn create_mcp_policies_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mcp_policies (
            credential_id TEXT NOT NULL,
            tag TEXT NOT NULL,
            authz_mode TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (credential_id, tag)
        )",
        [],
    )?;

    // Create index for efficient policy lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mcp_policies_credential
         ON mcp_policies(credential_id)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_database_creates_all_tables() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify all core tables exist
        let tables = vec![
            "records",
            "tags",
            "record_tags",
            "metadata",
            "sync_state",
            "mcp_sessions",
            "mcp_policies",
        ];

        for table in tables {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                    table
                ))
                .unwrap();
            let result: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
            assert_eq!(result, Some(table.to_string()), "Table {} should exist", table);
        }
    }

    #[test]
    fn test_initialize_database_enables_wal_mode() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify WAL mode is enabled
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_initialize_database_enables_foreign_keys() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify foreign keys are enabled
        let foreign_keys: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn test_initialize_database_sets_synchronous_normal() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify synchronous mode is NORMAL (value 1 or 2 depending on SQLite version)
        let synchronous: i32 = conn
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .unwrap();
        // NORMAL is 1 in newer SQLite, FULL is 2
        assert!(synchronous >= 1 && synchronous <= 2, "synchronous should be NORMAL (1) or FULL (2), got {}", synchronous);
    }

    #[test]
    fn test_initialize_database_sets_busy_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify busy timeout is set
        let busy_timeout: i32 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(busy_timeout, 5000);
    }

    #[test]
    fn test_records_table_has_all_columns() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Get table info for records
        let mut stmt = conn
            .prepare("PRAGMA table_info(records)")
            .unwrap();

        let columns: Vec<String> = stmt
            .query_map([], |row| Ok(row.get::<_, String>(1).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let expected_columns = vec![
            "id",
            "record_type",
            "encrypted_data",
            "nonce",
            "created_at",
            "updated_at",
            "updated_by",
            "version",
            "deleted",
        ];

        for col in expected_columns {
            assert!(
                columns.contains(&col.to_string()),
                "Column {} should exist in records table",
                col
            );
        }
    }

    #[test]
    fn test_records_table_has_indexes() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify indexes on records table
        let indexes = vec![
            "idx_records_type",
            "idx_records_updated",
            "idx_records_deleted",
        ];

        for index in indexes {
            let mut stmt = conn
                .prepare(&format!(
                    "SELECT name FROM sqlite_master WHERE type='index' AND name='{}'",
                    index
                ))
                .unwrap();
            let result: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
            assert_eq!(
                result,
                Some(index.to_string()),
                "Index {} should exist",
                index
            );
        }
    }

    #[test]
    fn test_tags_table_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify tags table has unique constraint on name
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='tags'")
            .unwrap();
        let sql: String = stmt.query_row([], |row| row.get(0)).unwrap();

        assert!(sql.contains("UNIQUE"), "tags.name should have UNIQUE constraint");
    }

    #[test]
    fn test_record_tags_junction_table_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify record_tags table has foreign keys
        let mut stmt = conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='record_tags'").unwrap();
        let sql: String = stmt.query_row([], |row| row.get(0)).unwrap();

        assert!(
            sql.contains("FOREIGN KEY"),
            "record_tags should have FOREIGN KEY constraints"
        );
        assert!(
            sql.contains("ON DELETE CASCADE"),
            "record_tags foreign keys should CASCADE on delete"
        );
    }

    #[test]
    fn test_sync_state_table_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify sync_state table has correct columns
        let mut stmt = conn
            .prepare("PRAGMA table_info(sync_state)")
            .unwrap();

        let columns: Vec<String> = stmt
            .query_map([], |row| Ok(row.get::<_, String>(1).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(
            columns.contains(&"record_id".to_string()),
            "sync_state should have record_id column"
        );
        assert!(
            columns.contains(&"cloud_updated_at".to_string()),
            "sync_state should have cloud_updated_at column"
        );
        assert!(
            columns.contains(&"sync_status".to_string()),
            "sync_state should have sync_status column"
        );
    }

    #[test]
    fn test_mcp_sessions_table_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify mcp_sessions table has required columns
        let mut stmt = conn
            .prepare("PRAGMA table_info(mcp_sessions)")
            .unwrap();

        let columns: Vec<String> = stmt
            .query_map([], |row| Ok(row.get::<_, String>(1).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let required_columns = vec![
            "id",
            "approved_credentials",
            "created_at",
            "last_activity",
            "ttl_seconds",
        ];

        for col in required_columns {
            assert!(
                columns.contains(&col.to_string()),
                "mcp_sessions should have {} column",
                col
            );
        }

        // Verify index exists
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_mcp_sessions_last_activity'")
            .unwrap();
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(result, Some("idx_mcp_sessions_last_activity".to_string()));
    }

    #[test]
    fn test_mcp_policies_table_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify mcp_policies table has composite primary key
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='mcp_policies'")
            .unwrap();
        let sql: String = stmt.query_row([], |row| row.get(0)).unwrap();

        assert!(
            sql.contains("PRIMARY KEY (credential_id, tag)"),
            "mcp_policies should have composite primary key on (credential_id, tag)"
        );

        // Verify index exists
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_mcp_policies_credential'")
            .unwrap();
        let result: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(result, Some("idx_mcp_policies_credential".to_string()));
    }

    #[test]
    fn test_initialize_database_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // First initialization
        let conn1 = initialize_database(&db_path).unwrap();
        drop(conn1);

        // Second initialization should not fail
        let conn2 = initialize_database(&db_path).unwrap();

        // Verify tables still exist
        let mut stmt = conn2
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table'")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert!(count >= 7, "Should have at least 7 tables");
    }

    #[test]
    fn test_metadata_table_key_value_structure() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = initialize_database(&db_path).unwrap();

        // Verify metadata table has key as primary key
        let mut stmt = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='metadata'")
            .unwrap();
        let sql: String = stmt.query_row([], |row| row.get(0)).unwrap();

        assert!(sql.contains("PRIMARY KEY"), "metadata should have primary key");
        assert!(
            sql.contains("key TEXT PRIMARY KEY"),
            "metadata.key should be the primary key"
        );
    }
}
