//! Metadata operations for key-value storage
//!
//! This module provides functions for managing metadata in the vault database.
//! All functions operate directly on database connections and don't require synchronization calls.

use anyhow::Result;
use rusqlite::Connection;

/// Set a metadata key-value pair
///
/// If the key already exists, it will be updated with the new value.
///
/// # Arguments
/// * `conn` - Mutable database connection
/// * `key` - Metadata key
/// * `value` - Metadata value
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::metadata::set_metadata;
/// use rusqlite::Connection;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut conn = Connection::open_in_memory()?;
/// set_metadata(&mut conn, "device_id", "abc123")?;
/// # Ok(())
/// # }
/// ```
pub fn set_metadata(conn: &mut Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}

/// Get metadata value by key
///
/// Returns `None` if the key does not exist.
///
/// # Arguments
/// * `conn` - Database connection
/// * `key` - Metadata key to retrieve
///
/// # Returns
/// * `Ok(Some(value))` - If key exists
/// * `Ok(None)` - If key doesn't exist
/// * `Err(...)` - On database error
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::metadata::get_metadata;
/// use rusqlite::Connection;
///
/// # fn main() -> anyhow::Result<()> {
/// let conn = Connection::open_in_memory()?;
/// let value = get_metadata(&conn, "device_id")?;
/// assert!(value.is_none());
/// # Ok(())
/// # }
/// ```
pub fn get_metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
    let result = conn.query_row("SELECT value FROM metadata WHERE key = ?1", [key], |row| {
        row.get(0)
    });

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Delete metadata value by key
///
/// # Arguments
/// * `conn` - Mutable database connection
/// * `key` - Metadata key to delete
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::metadata::{set_metadata, delete_metadata};
/// use rusqlite::Connection;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut conn = Connection::open_in_memory()?;
/// set_metadata(&mut conn, "temp_key", "temp_value")?;
/// delete_metadata(&mut conn, "temp_key")?;
/// # Ok(())
/// # }
/// ```
pub fn delete_metadata(conn: &mut Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM metadata WHERE key = ?1", [key])?;
    Ok(())
}

/// List all metadata keys matching a prefix
///
/// Returns all keys that start with the given prefix.
///
/// # Arguments
/// * `conn` - Database connection
/// * `prefix` - Key prefix to match (e.g., "sync:", "device:", etc.)
///
/// # Returns
/// Vector of matching key names
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::metadata::{set_metadata, list_metadata_keys};
/// use rusqlite::Connection;
///
/// # fn main() -> anyhow::Result<()> {
/// let mut conn = Connection::open_in_memory()?;
/// set_metadata(&mut conn, "sync:device1", "value1")?;
/// set_metadata(&mut conn, "sync:device2", "value2")?;
/// set_metadata(&mut conn, "other:key", "value3")?;
///
/// let keys = list_metadata_keys(&conn, "sync:")?;
/// assert_eq!(keys.len(), 2);
/// # Ok(())
/// # }
/// ```
pub fn list_metadata_keys(conn: &Connection, prefix: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT key FROM metadata WHERE key LIKE ?1")?;

    let mut keys = Vec::new();
    let mut rows = stmt.query([format!("{}%", prefix)])?;

    while let Some(row) = rows.next()? {
        keys.push(row.get(0)?);
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        // Create metadata table
        conn.execute(
            "CREATE TABLE metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_set_and_get_metadata() {
        let mut conn = create_test_db();

        set_metadata(&mut conn, "test-key", "test-value").unwrap();

        let value = get_metadata(&conn, "test-key").unwrap();
        assert_eq!(value, Some("test-value".to_string()));
    }

    #[test]
    fn test_get_metadata_not_exists() {
        let conn = create_test_db();

        let value = get_metadata(&conn, "nonexistent").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_set_metadata_update_existing() {
        let mut conn = create_test_db();

        set_metadata(&mut conn, "key", "value1").unwrap();
        set_metadata(&mut conn, "key", "value2").unwrap();

        let value = get_metadata(&conn, "key").unwrap();
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_delete_metadata() {
        let mut conn = create_test_db();

        set_metadata(&mut conn, "key", "value").unwrap();
        delete_metadata(&mut conn, "key").unwrap();

        let value = get_metadata(&conn, "key").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_list_metadata_keys() {
        let mut conn = create_test_db();

        set_metadata(&mut conn, "sync:device1", "value1").unwrap();
        set_metadata(&mut conn, "sync:device2", "value2").unwrap();
        set_metadata(&mut conn, "other:key", "value3").unwrap();

        let keys = list_metadata_keys(&conn, "sync:").unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"sync:device1".to_string()));
        assert!(keys.contains(&"sync:device2".to_string()));
    }

    #[test]
    fn test_list_metadata_keys_empty_prefix() {
        let mut conn = create_test_db();

        set_metadata(&mut conn, "key1", "value1").unwrap();
        set_metadata(&mut conn, "key2", "value2").unwrap();

        let keys = list_metadata_keys(&conn, "").unwrap();
        assert_eq!(keys.len(), 2);
    }
}
