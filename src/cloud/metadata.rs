//! Cloud Metadata Serialization
//!
//! Defines the metadata structures for cloud storage synchronization.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use base64::prelude::*;

/// Cloud metadata for synchronization
///
/// Contains format version, KDF nonce, device list, and record metadata.
/// Stored as `.metadata.json` in the cloud storage root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMetadata {
    /// Format version for compatibility checks
    pub format_version: String,
    /// KDF nonce used for key derivation (base64 encoded)
    pub kdf_nonce: String,
    /// Metadata creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp (optional, updated on changes)
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Metadata version number for conflict resolution
    pub metadata_version: u64,
    /// List of registered devices
    #[serde(default)]
    pub devices: Vec<DeviceInfo>,
    /// Record metadata indexed by record ID
    #[serde(default)]
    pub records: HashMap<String, RecordMetadata>,
}

impl Default for CloudMetadata {
    fn default() -> Self {
        Self {
            format_version: "1.0".to_string(),
            kdf_nonce: BASE64_STANDARD.encode([0u8; 32]),
            created_at: Utc::now(),
            updated_at: None,
            metadata_version: 1,
            devices: Vec::new(),
            records: HashMap::new(),
        }
    }
}

impl CloudMetadata {
    /// Increment the metadata version and update timestamp
    pub fn increment_version(&mut self) {
        self.metadata_version += 1;
        self.updated_at = Some(Utc::now());
    }
}

/// Device information for tracking synchronized devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier (platform-name-fingerprint)
    pub device_id: String,
    /// Platform identifier (macos, ios, linux, windows, etc.)
    pub platform: String,
    /// Human-readable device name
    pub device_name: String,
    /// Last synchronization timestamp
    pub last_seen: DateTime<Utc>,
    /// Number of sync operations performed
    pub sync_count: u64,
}

/// Record metadata for version tracking and conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    /// Record ID (matches local database)
    pub id: String,
    /// Record version number
    pub version: u64,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Device ID that last updated this record
    pub updated_by: String,
    /// Record type (password, note, etc.)
    #[serde(rename = "type")]
    pub type_: String,
    /// Checksum for data integrity verification
    pub checksum: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_metadata_default() {
        let metadata = CloudMetadata::default();
        assert_eq!(metadata.format_version, "1.0");
        assert_eq!(metadata.metadata_version, 1);
        assert!(metadata.updated_at.is_none());
        assert!(metadata.devices.is_empty());
        assert!(metadata.records.is_empty());
    }

    #[test]
    fn test_increment_version() {
        let mut metadata = CloudMetadata::default();
        assert_eq!(metadata.metadata_version, 1);

        metadata.increment_version();
        assert_eq!(metadata.metadata_version, 2);
        assert!(metadata.updated_at.is_some());
    }

    #[test]
    fn test_device_info_serialization() {
        let device = DeviceInfo {
            device_id: "test-device".to_string(),
            platform: "linux".to_string(),
            device_name: "Test Machine".to_string(),
            last_seen: Utc::now(),
            sync_count: 5,
        };

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.device_id, "test-device");
        assert_eq!(deserialized.platform, "linux");
        assert_eq!(deserialized.sync_count, 5);
    }

    #[test]
    fn test_record_metadata_serialization() {
        let record = RecordMetadata {
            id: "record-001".to_string(),
            version: 3,
            updated_at: Utc::now(),
            updated_by: "device-abc".to_string(),
            type_: "password".to_string(),
            checksum: "abc123".to_string(),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: RecordMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "record-001");
        assert_eq!(deserialized.version, 3);
        assert_eq!(deserialized.type_, "password");
    }
}
