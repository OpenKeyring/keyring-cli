//! Sync operations for vault records

use anyhow::Result;
use rusqlite::Connection;

use crate::db::models::{RecordType, StoredRecord, SyncState, SyncStatus};

/// Get sync state for a record
pub fn get_sync_state(conn: &Connection, record_id: &str) -> Result<Option<SyncState>> {
    let result = conn.query_row(
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
    conn: &mut Connection,
    record_id: &str,
    cloud_updated_at: Option<i64>,
    sync_status: SyncStatus,
) -> Result<()> {
    let sync_status_int = sync_status as i32;
    conn.execute(
        "INSERT OR REPLACE INTO sync_state (record_id, cloud_updated_at, sync_status) VALUES (?1, ?2, ?3)",
        (record_id, cloud_updated_at, sync_status_int),
    )?;
    Ok(())
}

/// Mark record as pending sync (when record is updated)
pub fn mark_record_pending(conn: &mut Connection, record_id: &str) -> Result<()> {
    set_sync_state(conn, record_id, None, SyncStatus::Pending)
}

/// Get sync statistics for all records
///
/// Returns aggregated counts of total records, and records by sync status.
pub fn get_sync_stats(conn: &Connection) -> Result<crate::db::SyncStats> {
    // Count total non-deleted records
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM records WHERE deleted = 0",
        [],
        |row| row.get(0),
    )?;

    // Count records by sync status
    let pending: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_state WHERE sync_status = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let synced: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_state WHERE sync_status = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let conflicts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_state WHERE sync_status = 2",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(crate::db::SyncStats {
        total,
        pending,
        synced,
        conflicts,
    })
}

/// Get all records with pending sync status
///
/// Returns records that have sync_status = Pending (0).
pub fn get_pending_records(conn: &Connection) -> Result<Vec<StoredRecord>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.record_type, r.encrypted_data, r.nonce, r.created_at, r.updated_at, r.version,
            GROUP_CONCAT(t.name, ',') as tag_names
         FROM records r
         LEFT JOIN record_tags rt ON r.id = rt.record_id
         LEFT JOIN tags t ON rt.tag_id = t.id
         INNER JOIN sync_state ss ON r.id = ss.record_id
         WHERE r.deleted = 0 AND ss.sync_status = 0
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

        let uuid = uuid::Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tags = tags_csv
            .map(|csv| {
                csv.split(',')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        let nonce = super::decode_nonce(&nonce_bytes).map_err(|_| {
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
            deleted: false,
        });
    }

    Ok(records)
}
