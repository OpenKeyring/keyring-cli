//! Vault operations for record management

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use uuid::Uuid;

use super::models::{Record, RecordType};

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

    /// List all non-deleted records with tags
    pub fn list_records(&self) -> Result<Vec<Record>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, record_type, encrypted_data, created_at, updated_at
         FROM records WHERE deleted = 0
         ORDER BY updated_at DESC"
        )?;

        let record_iter = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let record_type_str: String = row.get(1)?;
            let encrypted_data: String = row.get(2)?;
            let created_ts: i64 = row.get(3)?;
            let updated_ts: i64 = row.get(4)?;

            let uuid = Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok((
                uuid,
                record_type_str,
                encrypted_data,
                created_ts,
                updated_ts,
            ))
        })?;

        let mut records = Vec::new();
        for record in record_iter {
            let (uuid, record_type_str, encrypted_data, created_ts, updated_ts) = record?;

            // Load tags for this record
            let tags: Vec<String> = self
                .conn
                .prepare(
                    "SELECT t.name FROM tags t
             JOIN record_tags rt ON t.id = rt.tag_id
             WHERE rt.record_id = ?1",
                )?
                .query_map([&uuid.to_string()], |row| row.get(0))?
                .collect::<Result<Vec<_>, _>>()?;

            records.push(Record {
                id: uuid,
                record_type: RecordType::from(record_type_str),
                encrypted_data,
                name: String::new(),
                username: None,
                url: None,
                notes: None,
                tags,
                created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid created_at timestamp"))?,
                updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid updated_at timestamp"))?,
            });
        }

        Ok(records)
    }

    /// Get a specific record by ID with tags
    pub fn get_record(&self, id: &str) -> Result<Record> {
        // Validate UUID format first
        let uuid = Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID format: {}", e))?;

        let (_id_str, record_type_str, encrypted_data, created_ts, updated_ts) = self.conn.query_row(
            "SELECT id, record_type, encrypted_data, created_at, updated_at
         FROM records WHERE id = ?1 AND deleted = 0",
            &[id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )?;

        let record = Record {
            id: uuid,
            record_type: super::models::RecordType::from(record_type_str),
            encrypted_data,
            name: String::new(), // Will be decoded from encrypted_data
            username: None,
            url: None,
            notes: None,
            tags: vec![], // Will load below
            created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid created_at timestamp"))?,
            updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid updated_at timestamp"))?,
        };

        // Load tags
        let tags: Vec<String> = self
            .conn
            .prepare(
                "SELECT t.name FROM tags t
         JOIN record_tags rt ON t.id = rt.tag_id
         WHERE rt.record_id = ?1",
            )?
            .query_map(&[id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Record { tags, ..record })
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
