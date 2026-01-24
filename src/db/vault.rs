//! Vault operations for record management

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use uuid::Uuid;

use super::models::{RecordType, StoredRecord, SyncStatus, SyncState};

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
    ///
    /// Uses a single query with LEFT JOIN and GROUP_CONCAT to avoid N+1 query pattern.
    /// TODO: Decode encrypted data fields when crypto module is integrated
    pub fn list_records(&self) -> Result<Vec<StoredRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.record_type, r.encrypted_data, r.nonce, r.created_at, r.updated_at,
                GROUP_CONCAT(t.name, ',') as tag_names
         FROM records r
         LEFT JOIN record_tags rt ON r.id = rt.record_id
         LEFT JOIN tags t ON rt.tag_id = t.id
         WHERE r.deleted = 0
         GROUP BY r.id
         ORDER BY r.updated_at DESC"
        )?;

        let record_iter = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let record_type_str: String = row.get(1)?;
            let encrypted_data: Vec<u8> = row.get(2)?;
            let nonce_bytes: Vec<u8> = row.get(3)?;
            let created_ts: i64 = row.get(4)?;
            let updated_ts: i64 = row.get(5)?;
            let tags_csv: Option<String> = row.get(6)?;

            let uuid = Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let tags = tags_csv
                .map(|csv| csv.split(',').filter(|s| !s.is_empty()).map(String::from).collect())
                .unwrap_or_default();

            let nonce = decode_nonce(&nonce_bytes).map_err(|_| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid nonce length",
                )))
            })?;

            Ok((
                uuid,
                record_type_str,
                encrypted_data,
                nonce,
                created_ts,
                updated_ts,
                tags,
            ))
        })?;

        let mut records = Vec::new();
        for record in record_iter {
            let (uuid, record_type_str, encrypted_data, nonce, created_ts, updated_ts, tags) =
                record?;

            records.push(StoredRecord {
                id: uuid,
                record_type: RecordType::from(record_type_str),
                encrypted_data,
                nonce,
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
    pub fn get_record(&self, id: &str) -> Result<StoredRecord> {
        // Validate UUID format first
        let uuid = Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID format: {}", e))?;

        let (_id_str, record_type_str, encrypted_data, nonce_bytes, created_ts, updated_ts) =
            self.conn.query_row(
            "SELECT id, record_type, encrypted_data, nonce, created_at, updated_at
         FROM records WHERE id = ?1 AND deleted = 0",
            &[id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )?;

        let nonce = decode_nonce(&nonce_bytes)?;

        let record = StoredRecord {
            id: uuid,
            record_type: super::models::RecordType::from(record_type_str),
            encrypted_data,
            nonce,
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

        Ok(StoredRecord { tags, ..record })
    }

    /// Add a new record with tag support
    ///
    /// This method wraps the entire operation in a transaction for atomicity.
    /// If any part fails, all changes are rolled back.
    ///
    /// # Note on Nonce Field
    /// The nonce field is provided by the AES-256-GCM encryption process and
    /// stored alongside the encrypted payload.
    ///
    /// # Note on Device ID
    /// The `updated_by` field is currently set to "local" as a placeholder.
    /// In a future update, this should be replaced with the actual device ID
    /// from the device identification system.
    pub fn add_record(&mut self, record: &StoredRecord) -> Result<()> {
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
                record.nonce.as_slice(),
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

        // Mark record as pending sync
        self.mark_record_pending(&record.id.to_string())?;

        Ok(())
    }

    /// Set a metadata key-value pair
    ///
    /// If the key already exists, it will be updated with the new value.
    pub fn set_metadata(&mut self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    /// Get metadata value by key
    ///
    /// Returns `None` if the key does not exist.
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM metadata WHERE key = ?1",
            [key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update an existing record with version increment
    ///
    /// This method wraps the entire operation in a transaction for atomicity.
    /// If any part fails, all changes are rolled back.
    pub fn update_record(&mut self, record: &StoredRecord) -> Result<()> {
        // Start transaction for atomicity
        let tx = self.conn.unchecked_transaction()?;

        // Update record data
        let rows_affected = tx.execute(
            "UPDATE records
         SET encrypted_data = ?1, nonce = ?2, updated_at = ?3, version = version + 1
         WHERE id = ?4 AND deleted = 0",
            (
                &record.encrypted_data,
                record.nonce.as_slice(),
                record.updated_at.timestamp(),
                &record.id.to_string(),
            ),
        )?;

        // Verify record was updated
        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Record not found or deleted: {}", record.id));
        }

        // Update tags: remove old associations and add new ones
        tx.execute(
            "DELETE FROM record_tags WHERE record_id = ?1",
            [&record.id.to_string()],
        )?;

        // Deduplicate tags before processing
        let unique_tags: std::collections::HashSet<_> = record.tags.iter().collect();
        let record_id_str = record.id.to_string(); // Move outside loop to avoid repeated allocation

        for tag_name in unique_tags {
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

            tx.execute(
                "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
                (&record_id_str, tag_id),
            )?;
        }

        // Commit transaction
        tx.commit()?;

        // Mark record as pending sync
        self.mark_record_pending(&record.id.to_string())?;

        Ok(())
    }

    /// Delete a record (soft delete)
    pub fn delete_record(&mut self, _id: &str) -> Result<()> {
        // TODO: Implement soft delete
        anyhow::bail!("Vault::delete_record not yet implemented")
    }

    /// Get sync state for a record
    pub fn get_sync_state(&self, record_id: &str) -> Result<Option<SyncState>> {
        let result = self.conn.query_row(
            "SELECT cloud_updated_at, sync_status FROM sync_state WHERE record_id = ?1",
            [record_id],
            |row| {
                let cloud_updated_at: Option<i64> = row.get(0)?;
                let sync_status_int: i32 = row.get(1)?;
                let sync_status = match sync_status_int {
                    0 => SyncStatus::Pending,
                    1 => SyncStatus::Synced,
                    2 => SyncStatus::Conflict,
                    _ => SyncStatus::Pending,
                };
                Ok(SyncState {
                    record_id: record_id.to_string(),
                    cloud_updated_at,
                    sync_status,
                })
            },
        );

        match result {
            Ok(state) => Ok(Some(state)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set sync state for a record
    pub fn set_sync_state(
        &mut self,
        record_id: &str,
        cloud_updated_at: Option<i64>,
        sync_status: SyncStatus,
    ) -> Result<()> {
        let sync_status_int = sync_status as i32;
        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (record_id, cloud_updated_at, sync_status) VALUES (?1, ?2, ?3)",
            (record_id, cloud_updated_at, sync_status_int),
        )?;
        Ok(())
    }

    /// Mark record as pending sync (when record is updated)
    pub fn mark_record_pending(&mut self, record_id: &str) -> Result<()> {
        self.set_sync_state(record_id, None, SyncStatus::Pending)
    }

    /// Search records by pattern matching
    ///
    /// Currently searches the encrypted_data field. Once the crypto module is integrated,
    /// this should be updated to search the decrypted name field for better usability.
    ///
    /// Uses a single query with LEFT JOIN and GROUP_CONCAT to avoid N+1 query pattern.
    pub fn search_records(&self, query: &str) -> Result<Vec<StoredRecord>> {
        let pattern = format!("%{}%", query);

        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.record_type, r.encrypted_data, r.nonce, r.created_at, r.updated_at,
                GROUP_CONCAT(t.name, ',') as tag_names
         FROM records r
         LEFT JOIN record_tags rt ON r.id = rt.record_id
         LEFT JOIN tags t ON rt.tag_id = t.id
         WHERE r.deleted = 0 AND r.encrypted_data LIKE ?1
         GROUP BY r.id
         ORDER BY r.updated_at DESC"
        )?;

        let record_iter = stmt.query_map([&pattern], |row| {
            let id_str: String = row.get(0)?;
            let record_type_str: String = row.get(1)?;
            let encrypted_data: Vec<u8> = row.get(2)?;
            let nonce_bytes: Vec<u8> = row.get(3)?;
            let created_ts: i64 = row.get(4)?;
            let updated_ts: i64 = row.get(5)?;
            let tags_csv: Option<String> = row.get(6)?;

            let uuid = Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let tags = tags_csv
                .map(|csv| csv.split(',').filter(|s| !s.is_empty()).map(String::from).collect())
                .unwrap_or_default();

            let nonce = decode_nonce(&nonce_bytes).map_err(|_| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid nonce length",
                )))
            })?;

            Ok((
                uuid,
                record_type_str,
                encrypted_data,
                nonce,
                created_ts,
                updated_ts,
                tags,
            ))
        })?;

        let mut records = Vec::new();
        for record in record_iter {
            let (uuid, record_type_str, encrypted_data, nonce, created_ts, updated_ts, tags) =
                record?;

            records.push(StoredRecord {
                id: uuid,
                record_type: RecordType::from(record_type_str),
                encrypted_data,
                nonce,
                tags,
                created_at: chrono::DateTime::from_timestamp(created_ts, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid created_at timestamp"))?,
                updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
                    .ok_or_else(|| anyhow::anyhow!("Invalid updated_at timestamp"))?,
            });
        }

        Ok(records)
    }
}

fn decode_nonce(bytes: &[u8]) -> Result<[u8; 12]> {
    if bytes.len() != 12 {
        return Err(anyhow::anyhow!("Invalid nonce length: {}", bytes.len()));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(bytes);
    Ok(nonce)
}
