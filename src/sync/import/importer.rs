//! SyncImporter trait and implementations
//!
//! Defines the interface for importing sync records.

use crate::db::models::StoredRecord;
use crate::error::KeyringError;
use crate::sync::export::SyncRecord;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::fs;
use std::path::Path;

/// Trait for importing sync records
pub trait SyncImporter {
    /// Import a sync record from a file
    fn import_from_file(&self, path: &Path) -> Result<SyncRecord, KeyringError>;

    /// Import a sync record from JSON string
    fn import_from_json(&self, json: &str) -> Result<SyncRecord, KeyringError>;

    /// Convert a sync record to a database record
    fn sync_record_to_db(&self, sync_record: SyncRecord) -> Result<StoredRecord, KeyringError>;
}

/// JSON-based sync importer
pub struct JsonSyncImporter;

impl SyncImporter for JsonSyncImporter {
    fn import_from_file(&self, path: &Path) -> Result<SyncRecord, KeyringError> {
        let json = fs::read_to_string(path).map_err(|e| KeyringError::IoError(e.to_string()))?;
        self.import_from_json(&json)
    }

    fn import_from_json(&self, json: &str) -> Result<SyncRecord, KeyringError> {
        let sync_record: SyncRecord = serde_json::from_str(json)?;
        Ok(sync_record)
    }

    fn sync_record_to_db(&self, sync_record: SyncRecord) -> Result<StoredRecord, KeyringError> {
        let encrypted_data =
            STANDARD
                .decode(sync_record.encrypted_data)
                .map_err(|e| KeyringError::Crypto {
                    context: format!("Invalid encrypted_data encoding: {}", e),
                })?;

        let nonce_bytes = STANDARD
            .decode(sync_record.nonce)
            .map_err(|e| KeyringError::Crypto {
                context: format!("Invalid nonce encoding: {}", e),
            })?;

        let nonce = super::service::decode_nonce(&nonce_bytes)?;

        Ok(StoredRecord {
            id: uuid::Uuid::parse_str(&sync_record.id)?,
            record_type: sync_record.record_type,
            encrypted_data,
            nonce,
            tags: sync_record.metadata.tags,
            created_at: sync_record.created_at,
            updated_at: sync_record.updated_at,
            version: sync_record.version,
        })
    }
}
