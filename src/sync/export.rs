use crate::db::models::{RecordType, StoredRecord};
use crate::error::KeyringError;
use crate::types::SensitiveString;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub id: String,
    /// Version number for conflict detection (incremented on each update)
    pub version: u64,
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

/// Decrypted sync record with sensitive data wrapped in SensitiveString
///
/// This struct is used when handling decrypted data in sync operations.
/// The password field is wrapped in SensitiveString for automatic zeroization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDecryptedRecord {
    pub id: String,
    pub name: String,
    pub record_type: String,
    pub username: Option<String>,
    pub password: SensitiveString<String>, // Wrapped for auto-zeroization
    pub url: Option<String>,
    pub notes: Option<String>,
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
            version: record.version,
            record_type: record.record_type,
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

    /// Get metadata as a JSON string for security auditing
    ///
    /// This method is used to verify that metadata doesn't contain
    /// sensitive information like passkey, DEK, or master key.
    pub fn get_metadata_json(&self, metadata: &RecordMetadata) -> String {
        serde_json::to_string(metadata).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to create test StoredRecord
    fn create_test_stored_record(id: &str, version: u64, encrypted_data: Vec<u8>) -> StoredRecord {
        StoredRecord {
            id: uuid::Uuid::parse_str(id).unwrap(),
            record_type: RecordType::Password,
            encrypted_data,
            nonce: [0u8; 12],
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version,
        }
    }

    // SyncRecord struct tests
    #[test]
    fn test_sync_record_creation() {
        let record = SyncRecord {
            id: "test-id".to_string(),
            version: 1,
            record_type: RecordType::Password,
            encrypted_data: "encrypted-data".to_string(),
            nonce: "nonce123".to_string(),
            metadata: RecordMetadata {
                name: "Test Record".to_string(),
                tags: vec!["tag1".to_string()],
                platform: "test".to_string(),
                device_id: "device-1".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(record.id, "test-id");
        assert_eq!(record.version, 1);
        assert_eq!(record.metadata.name, "Test Record");
    }

    // RecordMetadata struct tests
    #[test]
    fn test_record_metadata_creation() {
        let metadata = RecordMetadata {
            name: "Test".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            platform: "linux".to_string(),
            device_id: "device-123".to_string(),
        };

        assert_eq!(metadata.name, "Test");
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.platform, "linux");
        assert_eq!(metadata.device_id, "device-123");
    }

    #[test]
    fn test_record_metadata_empty_tags() {
        let metadata = RecordMetadata {
            name: "Test".to_string(),
            tags: vec![],
            platform: "macos".to_string(),
            device_id: "device-456".to_string(),
        };

        assert!(metadata.tags.is_empty());
    }

    // SyncDecryptedRecord struct tests
    #[test]
    fn test_sync_decrypted_record_creation() {
        let record = SyncDecryptedRecord {
            id: "test-id".to_string(),
            name: "Test Record".to_string(),
            record_type: "password".to_string(),
            username: Some("user@example.com".to_string()),
            password: SensitiveString::new("secret".to_string()),
            url: Some("https://example.com".to_string()),
            notes: Some("Test notes".to_string()),
        };

        assert_eq!(record.id, "test-id");
        assert_eq!(record.name, "Test Record");
        assert!(record.username.is_some());
        assert_eq!(record.password.get(), "secret");
    }

    #[test]
    fn test_sync_decrypted_record_with_none_optional_fields() {
        let record = SyncDecryptedRecord {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            record_type: "password".to_string(),
            username: None,
            password: SensitiveString::new("secret".to_string()),
            url: None,
            notes: None,
        };

        assert!(record.username.is_none());
        assert!(record.url.is_none());
        assert!(record.notes.is_none());
    }

    // JsonSyncExporter::export_record tests
    #[test]
    fn test_export_record_success() {
        let exporter = JsonSyncExporter;

        let stored_record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440000", 1, vec![1, 2, 3, 4]);

        let result = exporter.export_record(&stored_record);

        assert!(result.is_ok());
        let sync_record = result.unwrap();

        assert_eq!(sync_record.id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(sync_record.version, 1);
        assert_eq!(sync_record.encrypted_data, "AQIDBA=="); // base64 of [1,2,3,4]
        assert_eq!(sync_record.nonce, "AAAAAAAAAAAAAAAA"); // base64 of [0u8; 12] (no padding)
        assert_eq!(sync_record.metadata.tags.len(), 2);
    }

    #[test]
    fn test_export_record_base64_encoding() {
        let exporter = JsonSyncExporter;

        // Test with known values
        let data = vec![0x00, 0x01, 0x02, 0x03];
        let stored_record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440001", 1, data);

        let result = exporter.export_record(&stored_record).unwrap();

        // Base64 of [0,1,2,3] is "AAECAw=="
        assert_eq!(result.encrypted_data, "AAECAw==");
    }

    #[test]
    fn test_export_record_preserves_tags() {
        let exporter = JsonSyncExporter;

        let mut stored_record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440002", 1, vec![1, 2, 3]);
        stored_record.tags = vec!["work".to_string(), "personal".to_string(), "important".to_string()];

        let result = exporter.export_record(&stored_record).unwrap();

        assert_eq!(result.metadata.tags.len(), 3);
        assert!(result.metadata.tags.contains(&"work".to_string()));
        assert!(result.metadata.tags.contains(&"personal".to_string()));
        assert!(result.metadata.tags.contains(&"important".to_string()));
    }

    #[test]
    fn test_export_record_sets_platform() {
        let exporter = JsonSyncExporter;

        let stored_record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440003", 1, vec![1, 2, 3]);

        let result = exporter.export_record(&stored_record).unwrap();

        // Platform should be set to current OS
        assert!(!result.metadata.platform.is_empty());
        assert_eq!(result.metadata.platform, std::env::consts::OS);
    }

    #[test]
    fn test_export_record_sets_device_id() {
        let exporter = JsonSyncExporter;

        let stored_record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440004", 1, vec![1, 2, 3]);

        let result = exporter.export_record(&stored_record).unwrap();

        assert_eq!(result.metadata.device_id, "unknown-device");
    }

    // JsonSyncExporter::export_multiple tests
    #[test]
    fn test_export_multiple_empty_list() {
        let exporter = JsonSyncExporter;

        let records: Vec<StoredRecord> = vec![];
        let result = exporter.export_multiple(&records);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_export_multiple_success() {
        let exporter = JsonSyncExporter;

        let records = vec![
            create_test_stored_record("550e8400-e29b-41d4-a716-446655440005", 1, vec![1, 2, 3]),
            create_test_stored_record("550e8400-e29b-41d4-a716-446655440006", 2, vec![4, 5, 6]),
            create_test_stored_record("550e8400-e29b-41d4-a716-446655440007", 3, vec![7, 8, 9]),
        ];

        let result = exporter.export_multiple(&records);

        assert!(result.is_ok());
        let sync_records = result.unwrap();

        assert_eq!(sync_records.len(), 3);
        assert_eq!(sync_records[0].version, 1);
        assert_eq!(sync_records[1].version, 2);
        assert_eq!(sync_records[2].version, 3);
    }

    #[test]
    fn test_export_multiple_preserves_order() {
        let exporter = JsonSyncExporter;

        let id1 = "550e8400-e29b-41d4-a716-446655440008";
        let id2 = "550e8400-e29b-41d4-a716-446655440009";
        let id3 = "550e8400-e29b-41d4-a716-446655440010";

        let records = vec![
            create_test_stored_record(id1, 1, vec![1]),
            create_test_stored_record(id2, 1, vec![2]),
            create_test_stored_record(id3, 1, vec![3]),
        ];

        let result = exporter.export_multiple(&records).unwrap();

        assert_eq!(result[0].id, id1);
        assert_eq!(result[1].id, id2);
        assert_eq!(result[2].id, id3);
    }

    // JsonSyncExporter::write_to_file tests
    #[test]
    fn test_write_to_file_success() {
        let exporter = JsonSyncExporter;

        let sync_record = SyncRecord {
            id: "test-id".to_string(),
            version: 1,
            record_type: RecordType::Password,
            encrypted_data: "encrypted".to_string(),
            nonce: "nonce123".to_string(),
            metadata: RecordMetadata {
                name: "Test".to_string(),
                tags: vec![],
                platform: "test".to_string(),
                device_id: "device1".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-record.json");

        let result = exporter.write_to_file(&sync_record, &file_path);

        assert!(result.is_ok());
        assert!(file_path.exists());

        // Verify the file contains valid JSON
        let content = fs::read_to_string(&file_path).unwrap();
        let parsed: SyncRecord = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.id, "test-id");
    }

    #[test]
    fn test_write_to_file_creates_valid_json() {
        let exporter = JsonSyncExporter;

        let sync_record = SyncRecord {
            id: "json-test".to_string(),
            version: 5,
            record_type: RecordType::Password,
            encrypted_data: "data".to_string(),
            nonce: "nonce".to_string(),
            metadata: RecordMetadata {
                name: "JSON Test".to_string(),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                platform: "linux".to_string(),
                device_id: "device-xyz".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("formatted.json");

        exporter.write_to_file(&sync_record, &file_path).unwrap();

        // Read and verify JSON is pretty-printed
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("\n")); // Pretty printed should have newlines
        assert!(content.contains("  ")); // Pretty printed should have indentation

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["id"], "json-test");
        assert_eq!(parsed["version"], 5);
    }

    #[test]
    fn test_write_to_file_overwrites_existing() {
        let exporter = JsonSyncExporter;

        let sync_record = SyncRecord {
            id: "overwrite-test".to_string(),
            version: 1,
            record_type: RecordType::Password,
            encrypted_data: "new-data".to_string(),
            nonce: "nonce".to_string(),
            metadata: RecordMetadata {
                name: "Overwrite".to_string(),
                tags: vec![],
                platform: "test".to_string(),
                device_id: "device1".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("overwrite.json");

        // Write first time
        fs::write(&file_path, "old content").unwrap();

        // Overwrite with sync record
        exporter.write_to_file(&sync_record, &file_path).unwrap();

        // Verify content was overwritten
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(!content.contains("old content"));
        assert!(content.contains("overwrite-test"));
    }

    // JsonSyncExporter::get_device_id tests
    #[test]
    fn test_get_device_id_returns_unknown_device() {
        let exporter = JsonSyncExporter;

        let device_id = exporter.get_device_id();

        assert!(device_id.is_ok());
        assert_eq!(device_id.unwrap(), "unknown-device");
    }

    // JsonSyncExporter::get_metadata_json tests
    #[test]
    fn test_get_metadata_json_valid_input() {
        let exporter = JsonSyncExporter;

        let metadata = RecordMetadata {
            name: "Test Record".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            platform: "linux".to_string(),
            device_id: "device-123".to_string(),
        };

        let json = exporter.get_metadata_json(&metadata);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "Test Record");
        assert_eq!(parsed["platform"], "linux");
        assert!(parsed["tags"].is_array());
    }

    #[test]
    fn test_get_metadata_json_empty_tags() {
        let exporter = JsonSyncExporter;

        let metadata = RecordMetadata {
            name: "Empty Tags".to_string(),
            tags: vec![],
            platform: "macos".to_string(),
            device_id: "device-456".to_string(),
        };

        let json = exporter.get_metadata_json(&metadata);

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["tags"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_get_metadata_json_special_characters() {
        let exporter = JsonSyncExporter;

        let metadata = RecordMetadata {
            name: "Test \"with\" quotes".to_string(),
            tags: vec!["tag&special".to_string()],
            platform: "linux".to_string(),
            device_id: "device-789".to_string(),
        };

        let json = exporter.get_metadata_json(&metadata);

        // Verify special characters are properly escaped in JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "Test \"with\" quotes");
        assert_eq!(parsed["tags"][0], "tag&special");
    }

    // Integration tests
    #[test]
    fn test_full_export_workflow() {
        let exporter = JsonSyncExporter;

        // Step 1: Create stored records
        let stored_records = vec![
            create_test_stored_record("550e8400-e29b-41d4-a716-446655440011", 1, vec![1, 2, 3]),
            create_test_stored_record("550e8400-e29b-41d4-a716-446655440012", 2, vec![4, 5, 6]),
        ];

        // Step 2: Export to sync records
        let sync_records = exporter.export_multiple(&stored_records).unwrap();
        assert_eq!(sync_records.len(), 2);

        // Step 3: Write to files
        let temp_dir = TempDir::new().unwrap();

        for sync_record in &sync_records {
            let file_path = temp_dir.path().join(format!("{}.json", sync_record.id));
            exporter.write_to_file(sync_record, &file_path).unwrap();
            assert!(file_path.exists());
        }

        // Verify files were created and contain valid data
        assert_eq!(temp_dir.path().read_dir().unwrap().count(), 2);
    }

    #[test]
    fn test_export_record_with_record_type() {
        let exporter = JsonSyncExporter;

        let mut record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440013", 1, vec![1]);
        record.record_type = RecordType::SshKey;

        let result = exporter.export_record(&record).unwrap();

        assert_eq!(result.record_type, RecordType::SshKey);
    }

    #[test]
    fn test_export_record_preserves_timestamps() {
        let exporter = JsonSyncExporter;

        let created_at = chrono::Utc::now() - chrono::Duration::hours(1);
        let updated_at = chrono::Utc::now();

        let mut record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440014", 1, vec![1]);
        record.created_at = created_at;
        record.updated_at = updated_at;

        let result = exporter.export_record(&record).unwrap();

        assert_eq!(result.created_at, created_at);
        assert_eq!(result.updated_at, updated_at);
    }

    #[test]
    fn test_export_record_with_nonce_encoding() {
        let exporter = JsonSyncExporter;

        let mut record = create_test_stored_record("550e8400-e29b-41d4-a716-446655440015", 1, vec![1]);
        record.nonce = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        let result = exporter.export_record(&record).unwrap();

        // Base64 of [1..12] is "AQIDBAUGBwgJCgsM"
        assert_eq!(result.nonce, "AQIDBAUGBwgJCgsM");
    }
}
