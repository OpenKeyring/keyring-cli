//! Vault operations for record management

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use super::models::Record;

/// Vault for managing encrypted password records
pub struct Vault {
    conn: Connection,
}

impl Vault {
    /// Open or create a vault at the specified path
    pub fn open(path: &Path, _master_password: &str) -> Result<Self> {
        let conn = super::schema::initialize_database(path)?;
        Ok(Self { conn })
    }

    /// List all records
    pub fn list_records(&self) -> Result<Vec<Record>> {
        // TODO: Implement listing
        Ok(vec![])
    }

    /// Get a specific record by ID
    pub fn get_record(&self, _id: &str) -> Result<Record> {
        // TODO: Implement retrieval
        anyhow::bail!("Vault::get_record not yet implemented")
    }

    /// Add a new record with tag support
    pub fn add_record(&mut self, record: &Record) -> Result<()> {
        self.conn.execute(
            "INSERT INTO records (id, record_type, encrypted_data, nonce, created_at, updated_at, updated_by, version, deleted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                record.id.to_string(),
                format!("{:?}", record.record_type).to_lowercase(),
                &record.encrypted_data,
                "",  // nonce placeholder - will be used with crypto module
                record.created_at.timestamp(),
                record.updated_at.timestamp(),
                "local",  // updated_by device
                1,  // version
                0,  // deleted
            ),
        )?;

        // Insert tags
        for tag_name in &record.tags {
            // Insert or get tag ID
            let tag_id: i64 = self.conn.query_row(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)
             RETURNING id",
                &[tag_name],
                |row| row.get(0),
            ).or_else(|_| {
                self.conn.query_row(
                    "SELECT id FROM tags WHERE name = ?1",
                    &[tag_name],
                    |row| row.get(0),
                )
            })?;

            // Link record to tag
            self.conn.execute(
                "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                (&record.id.to_string(), tag_id),
            )?;
        }

        Ok(())
    }

    /// Update an existing record
    pub fn update_record(&mut self, _record: &Record) -> Result<()> {
        // TODO: Implement update
        anyhow::bail!("Vault::update_record not yet implemented")
    }

    /// Delete a record (soft delete)
    pub fn delete_record(&mut self, _id: &str) -> Result<()> {
        // TODO: Implement soft delete
        anyhow::bail!("Vault::delete_record not yet implemented")
    }
}
