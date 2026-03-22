//! SyncImporterService implementation
//!
//! Service for importing sync records from directories.

use super::importer::{JsonSyncImporter, SyncImporter};
use crate::db::models::StoredRecord;
use crate::error::KeyringError;
use std::fs;
use std::path::Path;

/// Service for importing sync records
pub struct SyncImporterService {
    importer: Box<dyn SyncImporter>,
}

impl Default for SyncImporterService {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncImporterService {
    /// Create a new importer service with default JSON importer
    pub fn new() -> Self {
        Self {
            importer: Box::new(JsonSyncImporter),
        }
    }

    /// Import all JSON records from a directory
    pub fn import_records_from_dir(&self, dir: &Path) -> Result<Vec<StoredRecord>, KeyringError> {
        let mut records = Vec::new();

        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let sync_record = self.importer.import_from_file(&path)?;
                    let record = self.importer.sync_record_to_db(sync_record)?;
                    records.push(record);
                }
            }
        }

        Ok(records)
    }
}

/// Decode base64-decoded bytes into a 12-byte nonce array
pub fn decode_nonce(bytes: &[u8]) -> Result<[u8; 12], KeyringError> {
    if bytes.len() != 12 {
        return Err(KeyringError::Crypto {
            context: format!("Invalid nonce length: {}", bytes.len()),
        });
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(bytes);
    Ok(nonce)
}
