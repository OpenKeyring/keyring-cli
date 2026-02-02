use crate::db::models::StoredRecord;
use crate::error::KeyringError;
use crate::sync::export::SyncRecord;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::fs;
use std::path::Path;

pub trait SyncImporter {
    fn import_from_file(&self, path: &Path) -> Result<SyncRecord, KeyringError>;
    fn import_from_json(&self, json: &str) -> Result<SyncRecord, KeyringError>;
    fn sync_record_to_db(&self, sync_record: SyncRecord) -> Result<StoredRecord, KeyringError>;
}

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
        // In a real implementation, this would convert sync record to database record
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
        let nonce = decode_nonce(&nonce_bytes)?;

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

pub struct SyncImporterService {
    importer: Box<dyn SyncImporter>,
}

impl Default for SyncImporterService {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncImporterService {
    pub fn new() -> Self {
        Self {
            importer: Box::new(JsonSyncImporter),
        }
    }

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

fn decode_nonce(bytes: &[u8]) -> Result<[u8; 12], KeyringError> {
    if bytes.len() != 12 {
        return Err(KeyringError::Crypto {
            context: format!("Invalid nonce length: {}", bytes.len()),
        });
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(bytes);
    Ok(nonce)
}

// Helper function to create test SyncRecord
fn create_test_sync_record(id: &str, version: u64, encrypted_data: &str) -> SyncRecord {
    use crate::db::models::RecordType;
    SyncRecord {
        id: id.to_string(),
        version,
        record_type: RecordType::Password,
        encrypted_data: encrypted_data.to_string(),
        nonce: "AAAAAAAAAAAAAAAA".to_string(), // base64 of [0u8; 12]
        metadata: crate::sync::export::RecordMetadata {
            name: "Test Record".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            platform: "linux".to_string(),
            device_id: "device1".to_string(),
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // decode_nonce helper tests
    #[test]
    fn test_decode_nonce_valid_length() {
        let bytes = vec![0u8; 12];
        let result = decode_nonce(&bytes);

        assert!(result.is_ok());
        let nonce = result.unwrap();
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_decode_nonce_invalid_length_too_short() {
        let bytes = vec![0u8; 8];
        let result = decode_nonce(&bytes);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::Crypto { .. }));
    }

    #[test]
    fn test_decode_nonce_invalid_length_too_long() {
        let bytes = vec![0u8; 16];
        let result = decode_nonce(&bytes);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::Crypto { .. }));
    }

    #[test]
    fn test_decode_nonce_preserves_values() {
        let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let result = decode_nonce(&bytes).unwrap();

        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    // JsonSyncImporter::import_from_json tests
    #[test]
    fn test_import_from_json_valid() {
        let importer = JsonSyncImporter;

        let json = r#"{
            "id": "test-id-123",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AQIDBA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Test",
                "tags": ["tag1"],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        let result = importer.import_from_json(json);

        assert!(result.is_ok());
        let sync_record = result.unwrap();

        assert_eq!(sync_record.id, "test-id-123");
        assert_eq!(sync_record.version, 1);
        assert_eq!(sync_record.encrypted_data, "AQIDBA==");
    }

    #[test]
    fn test_import_from_json_invalid_json() {
        let importer = JsonSyncImporter;

        let invalid_json = "{ invalid json }";

        let result = importer.import_from_json(invalid_json);

        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_json_missing_required_field() {
        let importer = JsonSyncImporter;

        let json = r#"{
            "id": "test-id",
            "version": 1
        }"#; // Missing required fields

        let result = importer.import_from_json(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_json_with_tags() {
        let importer = JsonSyncImporter;

        let json = r#"{
            "id": "tags-test",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Tags Test",
                "tags": ["work", "personal", "important"],
                "platform": "macos",
                "device_id": "device2"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        let result = importer.import_from_json(json).unwrap();

        assert_eq!(result.metadata.tags.len(), 3);
        assert!(result.metadata.tags.contains(&"work".to_string()));
        assert!(result.metadata.tags.contains(&"personal".to_string()));
        assert!(result.metadata.tags.contains(&"important".to_string()));
    }

    // JsonSyncImporter::import_from_file tests
    #[test]
    fn test_import_from_file_success() {
        let importer = JsonSyncImporter;

        let json_content = r#"{
            "id": "file-test",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "File Test",
                "tags": [],
                "platform": "linux",
                "device_id": "device3"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-record.json");
        fs::write(&file_path, json_content).unwrap();

        let result = importer.import_from_file(&file_path);

        assert!(result.is_ok());
        let sync_record = result.unwrap();
        assert_eq!(sync_record.id, "file-test");
    }

    #[test]
    fn test_import_from_file_not_exists() {
        let importer = JsonSyncImporter;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");

        let result = importer.import_from_file(&file_path);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::IoError { .. }));
    }

    #[test]
    fn test_import_from_file_invalid_json() {
        let importer = JsonSyncImporter;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.json");
        fs::write(&file_path, "not valid json").unwrap();

        let result = importer.import_from_file(&file_path);

        assert!(result.is_err());
    }

    // JsonSyncImporter::sync_record_to_db tests
    #[test]
    fn test_sync_record_to_db_success() {
        let importer = JsonSyncImporter;

        let sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440000", 1, "AA==");

        let result = importer.sync_record_to_db(sync_record);

        assert!(result.is_ok());
        let stored_record = result.unwrap();

        assert_eq!(stored_record.id, uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        assert_eq!(stored_record.version, 1);
        assert_eq!(stored_record.encrypted_data, vec![0]);
        assert_eq!(stored_record.tags.len(), 2);
    }

    #[test]
    fn test_sync_record_to_db_invalid_base64_encoding() {
        let importer = JsonSyncImporter;

        let mut sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440001", 1, "AA==");
        sync_record.encrypted_data = "invalid base64!@#".to_string();

        let result = importer.sync_record_to_db(sync_record);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::Crypto { .. }));
    }

    #[test]
    fn test_sync_record_to_db_invalid_nonce_length() {
        let importer = JsonSyncImporter;

        let mut sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440002", 1, "AA==");
        // Nonce is only 8 bytes decoded instead of 12
        sync_record.nonce = "AAAAAAAA".to_string();

        let result = importer.sync_record_to_db(sync_record);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::Crypto { .. }));
    }

    #[test]
    fn test_sync_record_to_db_invalid_nonce_encoding() {
        let importer = JsonSyncImporter;

        let mut sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440003", 1, "AA==");
        sync_record.nonce = "invalid base64!@#".to_string();

        let result = importer.sync_record_to_db(sync_record);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeyringError::Crypto { .. }));
    }

    #[test]
    fn test_sync_record_to_db_invalid_uuid() {
        let importer = JsonSyncImporter;

        let sync_record = create_test_sync_record("not-a-valid-uuid", 1, "AA==");

        let result = importer.sync_record_to_db(sync_record);

        assert!(result.is_err());
    }

    #[test]
    fn test_sync_record_to_db_preserves_tags() {
        let importer = JsonSyncImporter;

        let sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440004", 1, "AA==");

        let result = importer.sync_record_to_db(sync_record).unwrap();

        assert_eq!(result.tags.len(), 2);
        assert!(result.tags.contains(&"tag1".to_string()));
        assert!(result.tags.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_sync_record_to_db_preserves_timestamps() {
        let importer = JsonSyncImporter;

        let created_at = chrono::Utc::now() - chrono::Duration::hours(1);
        let updated_at = chrono::Utc::now();

        let mut sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440005", 1, "AA==");
        sync_record.created_at = created_at;
        sync_record.updated_at = updated_at;

        let result = importer.sync_record_to_db(sync_record).unwrap();

        assert_eq!(result.created_at, created_at);
        assert_eq!(result.updated_at, updated_at);
    }

    #[test]
    fn test_sync_record_to_db_preserves_record_type() {
        let importer = JsonSyncImporter;

        let mut sync_record = create_test_sync_record("550e8400-e29b-41d4-a716-446655440006", 1, "AA==");
        sync_record.record_type = crate::db::models::RecordType::SshKey;

        let result = importer.sync_record_to_db(sync_record).unwrap();

        assert_eq!(result.record_type, crate::db::models::RecordType::SshKey);
    }

    // SyncImporterService::import_records_from_dir tests
    #[test]
    fn test_import_records_from_dir_success() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();

        // Create multiple JSON files
        let json1 = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440101",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Test 1",
                "tags": [],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        let json2 = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440102",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AQ==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Test 2",
                "tags": [],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        fs::write(temp_dir.path().join("record1.json"), json1).unwrap();
        fs::write(temp_dir.path().join("record2.json"), json2).unwrap();

        let result = service.import_records_from_dir(temp_dir.path());

        match &result {
            Ok(records) => {
                assert_eq!(records.len(), 2);
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_import_records_from_dir_nonexistent() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        let result = service.import_records_from_dir(&nonexistent_dir);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_import_records_from_dir_empty() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();
        // Create empty directory (temp_dir is already empty)

        let result = service.import_records_from_dir(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_import_records_from_dir_ignores_non_json() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();

        // Create mix of JSON and non-JSON files
        let valid_json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440104",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Test",
                "tags": [],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        fs::write(temp_dir.path().join("record1.json"), valid_json).unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "This is a readme").unwrap();
        fs::write(temp_dir.path().join("data.bin"), b"\x00\x01\x02\x03").unwrap();

        let result = service.import_records_from_dir(temp_dir.path());

        assert!(result.is_ok());
        let records = result.unwrap();
        // Only JSON file should be imported
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id.to_string(), "550e8400-e29b-41d4-a716-446655440104");
    }

    #[test]
    fn test_import_records_from_dir_mixed_files() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();

        let valid_json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440103",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Mixed Test",
                "tags": [],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        fs::write(temp_dir.path().join("valid.json"), valid_json).unwrap();
        fs::write(temp_dir.path().join("ignore.txt"), "ignore me").unwrap();

        let result = service.import_records_from_dir(temp_dir.path());

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id.to_string(), "550e8400-e29b-41d4-a716-446655440103");
    }

    #[test]
    fn test_import_records_from_dir_invalid_json_skipped() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();

        // Create invalid JSON file
        fs::write(temp_dir.path().join("invalid.json"), "not json").unwrap();

        let result = service.import_records_from_dir(temp_dir.path());

        // Should fail because invalid JSON causes error
        assert!(result.is_err());
    }

    // Integration tests
    #[test]
    fn test_full_import_workflow() {
        let service = SyncImporterService::new();

        // Step 1: Create export file (simulate export)
        let export_json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440100",
            "version": 5,
            "record_type": "password",
            "encrypted_data": "AAEC",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Full Workflow Test",
                "tags": ["test", "workflow"],
                "platform": "linux",
                "device_id": "workflow-device"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T01:00:00Z"
        }"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("export.json");
        fs::write(&file_path, export_json).unwrap();

        // Step 2: Import from directory
        let records = service.import_records_from_dir(temp_dir.path()).unwrap();

        assert_eq!(records.len(), 1);

        // Step 3: Verify converted record
        let record = &records[0];
        assert_eq!(record.version, 5);
        assert_eq!(record.encrypted_data, vec![0, 1, 2]);
        assert_eq!(record.tags.len(), 2);
        assert!(record.tags.contains(&"test".to_string()));
        assert!(record.tags.contains(&"workflow".to_string()));
    }

    #[test]
    fn test_import_export_roundtrip() {
        // This tests that export and import are compatible
        use crate::sync::export::JsonSyncExporter;
        use crate::sync::export::SyncExporter;

        let exporter = JsonSyncExporter;
        let service = SyncImporterService::new();

        // Step 1: Create a stored record manually and export it
        let stored_record = StoredRecord {
            id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440008").unwrap(),
            record_type: crate::db::models::RecordType::Password,
            encrypted_data: vec![10, 20, 30, 40],
            nonce: [0u8; 12],
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 3,
        };
        let sync_record = exporter.export_record(&stored_record).unwrap();

        // Step 2: Write to file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip.json");
        exporter.write_to_file(&sync_record, &file_path).unwrap();

        // Step 3: Import back
        let records = service.import_records_from_dir(temp_dir.path()).unwrap();

        assert_eq!(records.len(), 1);
        let imported = &records[0];

        // Step 4: Verify data matches
        assert_eq!(imported.id, stored_record.id);
        assert_eq!(imported.version, stored_record.version);
        assert_eq!(imported.encrypted_data, stored_record.encrypted_data);
        assert_eq!(imported.tags, stored_record.tags);
    }

    #[test]
    fn test_import_records_from_dir_nested_dirs_not_supported() {
        let service = SyncImporterService::new();

        let temp_dir = TempDir::new().unwrap();

        // Create nested directory (the implementation only reads top-level)
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();

        let valid_json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440105",
            "version": 1,
            "record_type": "password",
            "encrypted_data": "AA==",
            "nonce": "AAAAAAAAAAAAAAAA",
            "metadata": {
                "name": "Nested",
                "tags": [],
                "platform": "linux",
                "device_id": "device1"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;

        fs::write(nested_dir.join("nested.json"), valid_json).unwrap();

        let result = service.import_records_from_dir(temp_dir.path());

        // Should not import from nested directory (implementation only reads top-level)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
