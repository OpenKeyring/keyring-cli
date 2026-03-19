//! Search operations for vault records

use super::decode_nonce;
use crate::db::models::{RecordType, StoredRecord};
use anyhow::Result;
use rusqlite::Connection;
use uuid::Uuid;

/// Search records by pattern matching
///
/// Currently searches the encrypted_data field. Once the crypto module is integrated,
/// this should be updated to search the decrypted name field for better usability.
///
/// Uses a single query with LEFT JOIN and GROUP_CONCAT to avoid N+1 query pattern.
pub fn search_records(conn: &Connection, query: &str) -> Result<Vec<StoredRecord>> {
    let pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT r.id, r.record_type, r.encrypted_data, r.nonce, r.created_at, r.updated_at, r.version,
            GROUP_CONCAT(t.name, ',') as tag_names
     FROM records r
     LEFT JOIN record_tags rt ON r.id = rt.record_id
     LEFT JOIN tags t ON rt.tag_id = t.id
     WHERE r.deleted = 0 AND r.encrypted_data LIKE ?1
     GROUP BY r.id
     ORDER BY r.updated_at DESC",
    )?;

    let record_iter = stmt.query_map([&pattern], |row| {
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
        let (
            uuid,
            record_type_str,
            encrypted_data,
            nonce,
            created_ts,
            updated_ts,
            version,
            tags,
        ) = record?;

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
        });
    }

    Ok(records)
}

/// Find a record by its decrypted name
///
/// This function searches all non-deleted records, decrypts their names,
/// and returns the first record whose name matches the given name.
///
/// # Returns
/// * `Ok(Some(record))` - If a record with the matching name is found
/// * `Ok(None)` - If no record with the matching name exists
/// * `Err(...)` - If there's a database or decryption error
pub fn find_record_by_name(conn: &Connection, name: &str) -> Result<Option<StoredRecord>> {
    // Get all non-deleted records
    let records = super::record::list_records(conn)?;

    // Search through records to find one with matching name
    for record in records {
        // Try to parse the encrypted data as JSON to get the name
        // Note: This is a simplified approach since we don't have crypto context here
        // In production, this would need proper decryption
        if let Ok(payload_json) = std::str::from_utf8(&record.encrypted_data) {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(payload_json) {
                if let Some(record_name) = payload.get("name").and_then(|n| n.as_str()) {
                    if record_name == name {
                        return Ok(Some(record));
                    }
                }
            }
        }
    }

    Ok(None)
}
