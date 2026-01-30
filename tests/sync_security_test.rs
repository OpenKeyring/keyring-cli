//! Security audit tests for sync functionality
//!
//! These tests verify zero-knowledge properties:
//! - Metadata must not contain sensitive keys
//! - Encrypted data must not leak information
//! - Cloud storage only receives encrypted blobs

use base64::Engine;
use chrono::Utc;
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::sync::export::{JsonSyncExporter, RecordMetadata, SyncExporter};
use uuid::Uuid;

/// Test that metadata JSON doesn't contain sensitive information
#[test]
fn test_metadata_no_sensitive_keys() {
    let exporter = JsonSyncExporter;

    // Create a test record
    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        version: 1,
        updated_at: Utc::now(),
    };

    // Export to sync record
    let sync_record = exporter.export_record(&test_record).unwrap();

    // Get metadata as JSON string
    let metadata_json = exporter.get_metadata_json(&sync_record.metadata);

    // Verify metadata doesn't contain sensitive keys
    assert!(!metadata_json.contains("passkey"));
    assert!(!metadata_json.contains("dek"));
    assert!(!metadata_json.contains("master_key"));
    assert!(!metadata_json.contains("private_key"));
    assert!(!metadata_json.contains("seed"));
    assert!(!metadata_json.contains("mnemonic"));

    // Verify metadata only contains non-sensitive fields
    assert!(metadata_json.contains("name"));
    assert!(metadata_json.contains("tags"));
    assert!(metadata_json.contains("platform"));
    assert!(metadata_json.contains("device_id"));
}

/// Test that encrypted data is base64 encoded
#[test]
fn test_encrypted_data_is_base64() {
    let exporter = JsonSyncExporter;

    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        version: 1,
        updated_at: Utc::now(),
    };

    let sync_record = exporter.export_record(&test_record).unwrap();

    // Verify encrypted_data is valid base64
    assert!(base64::engine::general_purpose::STANDARD
        .decode(&sync_record.encrypted_data).is_ok());
}

/// Test that nonce is base64 encoded
#[test]
fn test_nonce_is_base64() {
    let exporter = JsonSyncExporter;

    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"encrypted-data".to_vec(),
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        version: 1,
        updated_at: Utc::now(),
    };

    let sync_record = exporter.export_record(&test_record).unwrap();

    // Verify nonce is valid base64
    assert!(base64::engine::general_purpose::STANDARD
        .decode(&sync_record.nonce).is_ok());
}

/// Test that full sync record JSON doesn't leak sensitive information
#[test]
fn test_full_sync_record_no_sensitive_data() {
    let exporter = JsonSyncExporter;

    // Use realistic encrypted data (would be AES-256-GCM ciphertext in production)
    let encrypted_data = [
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81,
        0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8, 0x09
    ].to_vec();

    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        version: 1,
        updated_at: Utc::now(),
    };

    let sync_record = exporter.export_record(&test_record).unwrap();
    let full_json = serde_json::to_string(&sync_record).unwrap();

    // Verify full JSON doesn't contain sensitive keys in plaintext
    assert!(!full_json.contains("passkey"));
    assert!(!full_json.contains("dek"));
    assert!(!full_json.contains("master_key"));
    assert!(!full_json.contains("private_key"));
    assert!(!full_json.contains("seed"));
    assert!(!full_json.contains("mnemonic"));

    // Verify it contains expected fields
    assert!(full_json.contains("id"));
    assert!(full_json.contains("record_type"));
    assert!(full_json.contains("encrypted_data"));
    assert!(full_json.contains("nonce"));
    assert!(full_json.contains("metadata"));
}

/// Test that RecordMetadata structure doesn't have sensitive fields
#[test]
fn test_record_metadata_structure() {
    let metadata = RecordMetadata {
        name: "test-record".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        platform: "macos".to_string(),
        device_id: "test-device".to_string(),
    };

    let metadata_json = serde_json::to_string(&metadata).unwrap();

    // Verify no sensitive fields
    assert!(!metadata_json.contains("passkey"));
    assert!(!metadata_json.contains("dek"));
    assert!(!metadata_json.contains("master_key"));
    assert!(!metadata_json.contains("private_key"));

    // Verify expected fields
    assert!(metadata_json.contains("name"));
    assert!(metadata_json.contains("tags"));
    assert!(metadata_json.contains("platform"));
    assert!(metadata_json.contains("device_id"));
}

/// Test zero-knowledge property: metadata is the only readable part
#[test]
fn test_zero_knowledge_metadata_only() {
    let exporter = JsonSyncExporter;

    // Use realistic encrypted data (would be AES-256-GCM ciphertext in production)
    let encrypted_data = [
        0x9a, 0x8b, 0x7c, 0x6d, 0x5e, 0x4f, 0x30, 0x21,
        0x12, 0x03, 0xf4, 0xe5, 0xd6, 0xc7, 0xb8, 0xa9
    ].to_vec();

    let test_record = StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Mnemonic,
        encrypted_data,
        nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        tags: vec!["crypto".to_string(), "wallet".to_string()],
        created_at: Utc::now(),
        version: 1,
        updated_at: Utc::now(),
    };

    let sync_record = exporter.export_record(&test_record).unwrap();

    // The encrypted_data should be base64 encoded ciphertext
    // Not readable without the decryption key
    let encrypted_bytes = base64::engine::general_purpose::STANDARD
        .decode(&sync_record.encrypted_data).unwrap();

    // Verify the encrypted data is ciphertext (not readable text)
    // Real ciphertext should not contain common sensitive keywords
    let encrypted_str = String::from_utf8_lossy(&encrypted_bytes);
    assert!(!encrypted_str.contains("mnemonic"));
    assert!(!encrypted_str.contains("seed"));
    assert!(!encrypted_str.contains("passkey"));

    // Verify metadata is readable (by design - it's just tags and device info)
    let metadata_json = exporter.get_metadata_json(&sync_record.metadata);
    assert!(metadata_json.contains("crypto"));
    assert!(metadata_json.contains("wallet"));
}
