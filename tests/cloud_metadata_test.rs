// tests/cloud/metadata_test.rs
use keyring_cli::cloud::metadata::{CloudMetadata, DeviceInfo, RecordMetadata};
use chrono::Utc;
use std::collections::HashMap;
use base64::prelude::*;

#[test]
fn test_metadata_serialization() {
    let device = DeviceInfo {
        device_id: "macos-MacBookPro-a1b2c3d4".to_string(),
        platform: "macos".to_string(),
        device_name: "MacBook Pro".to_string(),
        last_seen: Utc::now(),
        sync_count: 1,
    };

    let metadata = CloudMetadata {
        format_version: "1.0".to_string(),
        kdf_nonce: BASE64_STANDARD.encode([1u8; 32]),
        created_at: Utc::now(),
        updated_at: Some(Utc::now()),
        metadata_version: 1,
        devices: vec![device],
        records: HashMap::new(),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("kdf_nonce"));

    let deserialized: CloudMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.format_version, "1.0");
}

#[test]
fn test_metadata_version_increment() {
    let mut metadata = CloudMetadata::default();
    assert_eq!(metadata.metadata_version, 1);

    metadata.increment_version();
    assert_eq!(metadata.metadata_version, 2);
    assert!(metadata.updated_at.is_some());
}

#[test]
fn test_device_info_serialization() {
    let device = DeviceInfo {
        device_id: "test-device-123".to_string(),
        platform: "linux".to_string(),
        device_name: "Test Machine".to_string(),
        last_seen: Utc::now(),
        sync_count: 5,
    };

    let json = serde_json::to_string(&device).unwrap();
    let deserialized: DeviceInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.device_id, "test-device-123");
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
        checksum: "abc123def456".to_string(),
    };

    let json = serde_json::to_string(&record).unwrap();
    let deserialized: RecordMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "record-001");
    assert_eq!(deserialized.version, 3);
    assert_eq!(deserialized.type_, "password");
}
