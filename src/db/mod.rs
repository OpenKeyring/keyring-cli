//! Database layer with SQLite storage

pub mod schema;
pub mod models;
pub mod vault;
pub mod lock;

use crate::error::KeyringError;
use rusqlite::Connection;
use std::path::Path;

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
        self.conn.as_ref()
            .ok_or_else(|| KeyringError::Database { context: "Database not open".to_string() })
    }

    /// Get mutable connection for use with vault operations
    pub fn connection_mut(&mut self) -> Result<&mut Connection, KeyringError> {
        self.conn.as_mut()
            .ok_or_else(|| KeyringError::Database { context: "Database not open".to_string() })
    }
}
