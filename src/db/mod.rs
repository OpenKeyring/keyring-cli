//! Database layer with SQLite storage

pub mod lock;
pub mod migration;
pub mod migrations;
pub mod models;
pub mod schema;
pub mod vault;
pub mod wal;

use crate::error::KeyringError;
use rusqlite::Connection;
use std::path::Path;

// Re-exports for convenience
pub use lock::VaultLock;
pub use migration::{Migration, Migrator};
pub use models::{RecordType, StoredRecord, SyncState, SyncStats, SyncStatus};
pub use schema::initialize_database;
pub use vault::Vault;
pub use wal::{checkpoint, truncate};

/// High-level database manager
pub struct DatabaseManager {
    conn: Option<Connection>,
    db_path: std::path::PathBuf,
}

impl DatabaseManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, KeyringError> {
        let db_path = db_path.as_ref().to_path_buf();
        Ok(Self {
            conn: None,
            db_path,
        })
    }

    /// Open or create the database
    pub fn open(&mut self) -> Result<(), KeyringError> {
        self.conn = Some(schema::initialize_database(&self.db_path)?);
        Ok(())
    }

    /// Close the database connection
    pub fn close(&mut self) -> Result<(), KeyringError> {
        self.conn = None;
        Ok(())
    }

    /// Get the connection for use with vault operations
    pub fn connection(&self) -> Result<&Connection, KeyringError> {
        self.conn.as_ref().ok_or_else(|| KeyringError::Database {
            context: "Database not open".to_string(),
        })
    }

    /// Get mutable connection for use with vault operations
    pub fn connection_mut(&mut self) -> Result<&mut Connection, KeyringError> {
        self.conn.as_mut().ok_or_else(|| KeyringError::Database {
            context: "Database not open".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_manager_new() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = DatabaseManager::new(&db_path).unwrap();
        assert_eq!(manager.db_path, db_path);
        assert!(manager.conn.is_none());
    }

    #[test]
    fn test_database_manager_open() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut manager = DatabaseManager::new(&db_path).unwrap();
        assert!(manager.open().is_ok());

        // Verify database file was created
        assert!(db_path.exists());

        // Verify connection is now available
        assert!(manager.connection().is_ok());
        assert!(manager.connection_mut().is_ok());
    }

    #[test]
    fn test_database_manager_close() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut manager = DatabaseManager::new(&db_path).unwrap();
        manager.open().unwrap();
        assert!(manager.connection().is_ok());

        manager.close().unwrap();
        // After close, connection should not be available
        assert!(manager.connection().is_err());
        assert!(manager.connection_mut().is_err());
    }

    #[test]
    fn test_database_manager_connection_without_open_fails() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut manager = DatabaseManager::new(&db_path).unwrap();
        // Without calling open(), connection should fail
        assert!(manager.connection().is_err());
        assert!(manager.connection_mut().is_err());

        let err = manager.connection().unwrap_err();
        assert!(matches!(err, KeyringError::Database { .. }));
    }

    #[test]
    fn test_database_manager_reopen() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let mut manager = DatabaseManager::new(&db_path).unwrap();

        // First open
        manager.open().unwrap();
        let _conn1 = manager.connection().unwrap();
        // Connection reference goes out of scope here

        // Close and reopen
        manager.close().unwrap();
        manager.open().unwrap();

        // Should have a valid connection again
        assert!(manager.connection().is_ok());
        assert!(manager.connection_mut().is_ok());
    }

    #[test]
    fn test_database_manager_path_handling() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Test with PathBuf
        let path_buf = temp_dir.path().to_path_buf();
        let manager = DatabaseManager::new(&path_buf).unwrap();
        assert_eq!(manager.db_path, path_buf);

        // Test with &str
        let manager2 = DatabaseManager::new(temp_dir.path()).unwrap();
        assert_eq!(manager2.db_path, temp_dir.path());
    }
}
