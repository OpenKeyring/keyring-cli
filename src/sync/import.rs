use crate::error::KeyringError;
use crate::db::models::Record;
use crate::sync::export::SyncRecord;
use std::fs;
use std::path::Path;

pub trait SyncImporter {
    fn import_from_file(&self, path: &Path) -> Result<SyncRecord, KeyringError>;
    fn import_from_json(&self, json: &str) -> Result<SyncRecord, KeyringError>;
    fn sync_record_to_db(&self, sync_record: SyncRecord) -> Result<Record, KeyringError>;
}

pub struct JsonSyncImporter;

impl SyncImporter for JsonSyncImporter {
    fn import_from_file(&self, path: &Path) -> Result<SyncRecord, KeyringError> {
        let json = fs::read_to_string(path)
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        self.import_from_json(&json)
    }

    fn import_from_json(&self, json: &str) -> Result<SyncRecord, KeyringError> {
        let sync_record: SyncRecord = serde_json::from_str(json)
            .map_err(|e| KeyringError::SerializationError(e.to_string()))?;

        Ok(sync_record)
    }

    fn sync_record_to_db(&self, sync_record: SyncRecord) -> Result<Record, KeyringError> {
        // In a real implementation, this would convert sync record to database record
        Ok(Record {
            id: uuid::Uuid::parse_str(&sync_record.id)?,
            record_type: sync_record.record_type,
            encrypted_data: sync_record.encrypted_data,
            name: sync_record.metadata.name,
            username: None,
            url: None,
            notes: None,
            tags: sync_record.metadata.tags,
            created_at: sync_record.created_at,
            updated_at: sync_record.updated_at,
        })
    }
}

pub struct SyncImporterService {
    importer: Box<dyn SyncImporter>,
}

impl SyncImporterService {
    pub fn new() -> Self {
        Self {
            importer: Box::new(JsonSyncImporter),
        }
    }

    pub fn import_records_from_dir(&self, dir: &Path) -> Result<Vec<Record>, KeyringError> {
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