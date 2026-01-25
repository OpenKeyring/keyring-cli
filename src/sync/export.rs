use crate::db::models::{RecordType, StoredRecord};
use crate::error::KeyringError;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub id: String,
    pub record_type: RecordType,
    pub encrypted_data: String,
    pub nonce: String,
    pub metadata: RecordMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    pub name: String,
    pub tags: Vec<String>,
    pub platform: String,
    pub device_id: String,
}

pub trait SyncExporter {
    fn export_record(&self, record: &StoredRecord) -> Result<SyncRecord, KeyringError>;
    fn export_multiple(&self, records: &[StoredRecord]) -> Result<Vec<SyncRecord>, KeyringError>;
    fn write_to_file(&self, record: &SyncRecord, path: &Path) -> Result<(), KeyringError>;
}

pub struct JsonSyncExporter;

impl SyncExporter for JsonSyncExporter {
    fn export_record(&self, record: &StoredRecord) -> Result<SyncRecord, KeyringError> {
        let sync_record = SyncRecord {
            id: record.id.to_string(),
            record_type: record.record_type.clone(),
            encrypted_data: STANDARD.encode(&record.encrypted_data),
            nonce: STANDARD.encode(record.nonce),
            metadata: RecordMetadata {
                name: String::new(),
                tags: record.tags.clone(),
                platform: std::env::consts::OS.to_string(),
                device_id: self.get_device_id()?,
            },
            created_at: record.created_at,
            updated_at: record.updated_at,
        };

        Ok(sync_record)
    }

    fn export_multiple(&self, records: &[StoredRecord]) -> Result<Vec<SyncRecord>, KeyringError> {
        records
            .iter()
            .map(|record| self.export_record(record))
            .collect()
    }

    fn write_to_file(&self, record: &SyncRecord, path: &Path) -> Result<(), KeyringError> {
        let json = serde_json::to_string_pretty(record)?;

        fs::write(path, json).map_err(|e| KeyringError::IoError(e.to_string()))?;

        Ok(())
    }
}

impl JsonSyncExporter {
    fn get_device_id(&self) -> Result<String, KeyringError> {
        // In a real implementation, this would read from device config
        Ok("unknown-device".to_string())
    }
}
