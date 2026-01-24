//! Vault operations for record management

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use super::models::Record;

/// Vault for managing encrypted password records
pub struct Vault {
    /// Database connection (public for testing purposes)
    pub conn: Connection,
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
    ///
    /// This method wraps the entire operation in a transaction for atomicity.
    /// If any part fails, all changes are rolled back.
    ///
    /// # Note on Nonce Field
    /// The nonce field is currently set to an empty string as a placeholder.
    /// This will be addressed when the crypto module is integrated, as the actual
    /// nonce should come from the AES-256-GCM encryption process that happens
    /// before calling this method.
    ///
    /// # Note on Device ID
    /// The `updated_by` field is currently set to "local" as a placeholder.
    /// In a future update, this should be replaced with the actual device ID
    /// from the device identification system.
    pub fn add_record(&mut self, record: &Record) -> Result<()> {
        // Start transaction for atomicity
        let tx = self.conn.unchecked_transaction()?;

        // Insert record
        let record_type_str = record.record_type.to_db_string();
        let rows_affected = tx.execute(
            "INSERT INTO records (id, record_type, encrypted_data, nonce, created_at, updated_at, updated_by, version, deleted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                record.id.to_string(),
                record_type_str,
                &record.encrypted_data,
                "",  // nonce placeholder - see function docs
                record.created_at.timestamp(),
                record.updated_at.timestamp(),
                "local",  // updated_by device placeholder - see function docs
                1,  // version
                0,  // deleted (active record)
            ),
        )?;

        // Verify record was inserted
        if rows_affected != 1 {
            return Err(anyhow::anyhow!("Failed to insert record: expected 1 row affected, got {}", rows_affected));
        }

        // Deduplicate tags before processing
        let unique_tags: std::collections::HashSet<_> = record.tags.iter().collect();
        let record_id_str = record.id.to_string(); // Move outside loop to avoid repeated allocation

        // Insert tags
        for tag_name in unique_tags {
            // Insert or get tag ID
            let tag_id: i64 = tx.query_row(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)
             RETURNING id",
                &[tag_name],
                |row| row.get(0),
            ).or_else(|_| {
                tx.query_row(
                    "SELECT id FROM tags WHERE name = ?1",
                    &[tag_name],
                    |row| row.get(0),
                )
            })?;

            // Link record to tag
            tx.execute(
                "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                (&record_id_str, tag_id),
            )?;
        }

        // Commit transaction
        tx.commit()?;

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
