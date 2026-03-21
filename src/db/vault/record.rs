//! Record CRUD operations for the vault
//!
//! This module contains functions for creating, reading, updating, and deleting
//! password records in the vault database.

use super::decode_nonce;
use crate::db::models::{RecordType, StoredRecord, SyncStatus};
use anyhow::Result;
use rusqlite::Connection;
use uuid::Uuid;

/// List all non-deleted records with tags
///
/// Uses a single query with LEFT JOIN and GROUP_CONCAT to avoid N+1 query pattern.
/// Note: Returns encrypted records. Use get_record_decrypted() for decrypted records.
pub fn list_records(conn: &Connection) -> Result<Vec<StoredRecord>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.record_type, r.encrypted_data, r.nonce, r.created_at, r.updated_at, r.version,
            GROUP_CONCAT(t.name, ',') as tag_names
     FROM records r
     LEFT JOIN record_tags rt ON r.id = rt.record_id
     LEFT JOIN tags t ON rt.tag_id = t.id
     WHERE r.deleted = 0
     GROUP BY r.id
     ORDER BY r.updated_at DESC",
    )?;

    let record_iter = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let record_type_str: String = row.get(1)?;
        let encrypted_data: Vec<u8> = row.get(2)?;
        let nonce_bytes: Vec<u8> = row.get(3)?;
        let created_ts: i64 = row.get(4)?;
        let updated_ts: i64 = row.get(5)?;
        let version: i64 = row.get(6)?;
        let tags_csv: Option<String> = row.get(7)?;

        let uuid = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tags = tags_csv
            .map(|csv| {
                csv.split(',')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
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
            version as u64,
            tags,
        ))
    })?;

    let mut records = Vec::new();
    for record in record_iter {
        let (uuid, record_type_str, encrypted_data, nonce, created_ts, updated_ts, version, tags) =
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
            version,
            deleted: false, // WHERE deleted=0 already filters
        });
    }

    Ok(records)
}

/// Get a specific record by ID with tags
pub fn get_record(conn: &Connection, id: &str) -> Result<StoredRecord> {
    // Validate UUID format first
    let uuid = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid UUID format: {}", e))?;

    let (_id_str, record_type_str, encrypted_data, nonce_bytes, created_ts, updated_ts, version) =
        conn.query_row(
            "SELECT id, record_type, encrypted_data, nonce, created_at, updated_at, version
     FROM records WHERE id = ?1 AND deleted = 0",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            },
        )?;

    let nonce = decode_nonce(&nonce_bytes)?;

    let record = StoredRecord {
        id: uuid,
        record_type: RecordType::from(record_type_str),
        encrypted_data,
        nonce,
        tags: vec![], // Will load below
        created_at: chrono::DateTime::from_timestamp(created_ts, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid created_at timestamp"))?,
        updated_at: chrono::DateTime::from_timestamp(updated_ts, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid updated_at timestamp"))?,
        version: version as u64,
        deleted: false, // WHERE deleted=0 already filters
    };

    // Load tags
    let tags: Vec<String> = conn
        .prepare(
            "SELECT t.name FROM tags t
     JOIN record_tags rt ON t.id = rt.tag_id
     WHERE rt.record_id = ?1",
        )?
        .query_map([id], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(StoredRecord { tags, ..record })
}

/// Add a new record with tag support
///
/// This function wraps the entire operation in a transaction for atomicity.
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
pub fn add_record(conn: &mut Connection, record: &StoredRecord) -> Result<()> {
    // Start transaction for atomicity
    let tx = conn.unchecked_transaction()?;

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
        return Err(anyhow::anyhow!(
            "Failed to insert record: expected 1 row affected, got {}",
            rows_affected
        ));
    }

    // Deduplicate tags before processing
    let unique_tags: std::collections::HashSet<_> = record.tags.iter().collect();
    let record_id_str = record.id.to_string(); // Move outside loop to avoid repeated allocation

    // Insert tags
    for tag_name in unique_tags {
        // Insert or get tag ID
        let tag_id: i64 = tx
            .query_row(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)
         RETURNING id",
                [tag_name],
                |row| row.get(0),
            )
            .or_else(|_| {
                tx.query_row("SELECT id FROM tags WHERE name = ?1", [tag_name], |row| {
                    row.get(0)
                })
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
    super::sync::set_sync_state(conn, &record.id.to_string(), None, SyncStatus::Pending)?;

    Ok(())
}

/// Update an existing record with version increment
///
/// This function wraps the entire operation in a transaction for atomicity.
/// If any part fails, all changes are rolled back.
pub fn update_record(conn: &mut Connection, record: &StoredRecord) -> Result<()> {
    // Start transaction for atomicity
    let tx = conn.unchecked_transaction()?;

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
        return Err(anyhow::anyhow!(
            "Record not found or deleted: {}",
            record.id
        ));
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
        let tag_id: i64 = tx
            .query_row(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)
         RETURNING id",
                [tag_name],
                |row| row.get(0),
            )
            .or_else(|_| {
                tx.query_row("SELECT id FROM tags WHERE name = ?1", [tag_name], |row| {
                    row.get(0)
                })
            })?;

        tx.execute(
            "INSERT OR IGNORE INTO record_tags (record_id, tag_id) VALUES (?1, ?2)",
            (&record_id_str, tag_id),
        )?;
    }

    // Commit transaction
    tx.commit()?;

    // Mark record as pending sync
    super::sync::set_sync_state(conn, &record.id.to_string(), None, SyncStatus::Pending)?;

    Ok(())
}

/// Delete a record (soft delete)
///
/// Marks the record as deleted (deleted=1) and updates the updated_at timestamp.
/// The record data is retained in the database for potential recovery and sync purposes.
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - The UUID of the record to delete
///
/// # Returns
/// * `Ok(())` if the record was successfully marked as deleted
/// * `Err(...)` if the record doesn't exist or database error occurs
pub fn delete_record(conn: &mut Connection, id: &str) -> Result<()> {
    let rows_affected = conn.execute(
        "UPDATE records
         SET deleted = 1, updated_at = ?1
         WHERE id = ?2 AND deleted = 0",
        (chrono::Utc::now().timestamp(), id),
    )?;

    if rows_affected == 0 {
        return Err(anyhow::anyhow!(
            "Record not found or already deleted: {}",
            id
        ));
    }

    // Mark record as pending sync (for deletion propagation)
    super::sync::set_sync_state(conn, id, None, SyncStatus::Pending)?;

    Ok(())
}

/// Restore a soft-deleted record
///
/// Marks the record as active (deleted=0) and updates the timestamp.
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - The UUID of the record to restore
///
/// # Returns
/// * `Ok(())` if the record was successfully restored
/// * `Err(...)` if the record doesn't exist or is not deleted
pub(crate) fn restore_record(conn: &mut Connection, id: &str) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let rows = conn.execute(
        "UPDATE records SET deleted = 0, updated_at = ?1 WHERE id = ?2 AND deleted = 1",
        rusqlite::params![now, id],
    )?;
    if rows == 0 {
        anyhow::bail!("Record not found or not deleted: {}", id);
    }

    // Mark record as pending sync (for restore propagation)
    super::sync::set_sync_state(conn, id, None, SyncStatus::Pending)?;

    Ok(())
}

/// Permanently delete a record from the database
///
/// Removes the record and all associated data (tags, sync state) from the database.
/// This operation is irreversible.
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - The UUID of the record to permanently delete
pub(crate) fn permanently_delete_record(conn: &mut Connection, id: &str) -> Result<()> {
    let tx = conn.transaction()?;
    // Delete tag associations first
    tx.execute("DELETE FROM record_tags WHERE record_id = ?1", [id])?;
    // Delete sync state
    tx.execute("DELETE FROM sync_state WHERE record_id = ?1", [id])?;
    // Delete the record
    let rows = tx.execute("DELETE FROM records WHERE id = ?1", [id])?;
    tx.commit()?;
    if rows == 0 {
        anyhow::bail!("Record not found: {}", id);
    }
    Ok(())
}

/// List all soft-deleted records
///
/// Returns all records marked as deleted (deleted=1), with tags loaded.
/// Uses the same pattern as `list_records()` but filters for deleted records.
pub(crate) fn list_deleted_records(conn: &Connection) -> Result<Vec<StoredRecord>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.record_type, r.encrypted_data, r.nonce,
                r.created_at, r.updated_at, r.version,
                GROUP_CONCAT(t.name, ',') as tags
         FROM records r
         LEFT JOIN record_tags rt ON r.id = rt.record_id
         LEFT JOIN tags t ON rt.tag_id = t.id
         WHERE r.deleted = 1
         GROUP BY r.id
         ORDER BY r.updated_at DESC",
    )?;

    let record_iter = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let record_type_str: String = row.get(1)?;
        let encrypted_data: Vec<u8> = row.get(2)?;
        let nonce_bytes: Vec<u8> = row.get(3)?;
        let created_ts: i64 = row.get(4)?;
        let updated_ts: i64 = row.get(5)?;
        let version: i64 = row.get(6)?;
        let tags_csv: Option<String> = row.get(7)?;

        let uuid = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tags = tags_csv
            .map(|csv| {
                csv.split(',')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
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
            version as u64,
            tags,
        ))
    })?;

    let mut records = Vec::new();
    for record in record_iter {
        let (uuid, record_type_str, encrypted_data, nonce, created_ts, updated_ts, version, tags) =
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
            version,
            deleted: true,
        });
    }

    Ok(records)
}

/// Empty trash: permanently delete all soft-deleted records
///
/// Removes all records marked as deleted, along with their tag associations
/// and sync state. Returns the number of records deleted.
pub(crate) fn empty_trash(conn: &mut Connection) -> Result<usize> {
    // Get IDs of deleted records for cleanup
    let ids: Vec<String> = conn
        .prepare("SELECT id FROM records WHERE deleted = 1")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    if ids.is_empty() {
        return Ok(0);
    }

    let tx = conn.transaction()?;
    for id in &ids {
        tx.execute("DELETE FROM record_tags WHERE record_id = ?1", [id])?;
        tx.execute("DELETE FROM sync_state WHERE record_id = ?1", [id])?;
    }
    let count = tx.execute("DELETE FROM records WHERE deleted = 1", [])?;
    tx.commit()?;

    Ok(count)
}
