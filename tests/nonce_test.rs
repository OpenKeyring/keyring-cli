//! Tests for nonce verification on sync operations
//!
//! These tests verify that the NonceValidator properly:
//! - Detects matching nonces
//! - Detects mismatched nonces (tampering detected)
//! - Provides appropriate recovery strategies
//! - Handles user interaction for resolution

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::sync::export::SyncRecord;
use keyring_cli::sync::nonce_validator::{NonceStatus, NonceValidator, RecoveryStrategy};
use uuid::Uuid;

#[test]
fn test_validate_matching_nonce() {
    let validator = NonceValidator::new();

    // Create a test record with a specific nonce
    let nonce = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let local_record = create_test_record_with_nonce(nonce);

    // Create a sync record with the same nonce
    let sync_record = create_sync_record_with_nonce(nonce);

    // Validate should return Ok with NonceStatus::Valid
    let result = validator.validate(&local_record, &sync_record);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), NonceStatus::Valid);
}

#[test]
fn test_validate_mismatched_nonce() {
    let validator = NonceValidator::new();

    // Create a local record with one nonce
    let local_nonce = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let local_record = create_test_record_with_nonce(local_nonce);

    // Create a sync record with a different nonce (simulating tampering)
    let tampered_nonce = [99u8, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88];
    let sync_record = create_sync_record_with_nonce(tampered_nonce);

    // Validate should return Ok with NonceStatus::Mismatch
    let result = validator.validate(&local_record, &sync_record);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), NonceStatus::Mismatch);
}

#[test]
fn test_validate_with_corrupted_nonce() {
    let validator = NonceValidator::new();

    // Create a local record
    let nonce = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let local_record = create_test_record_with_nonce(nonce);

    // Create a sync record with corrupted nonce (wrong length)
    let mut sync_record = create_sync_record_with_nonce(nonce);
    sync_record.nonce = STANDARD.encode(&[1u8, 2, 3]); // Only 3 bytes instead of 12

    // Validate should return an error
    let result = validator.validate(&local_record, &sync_record);
    assert!(result.is_err());
}

#[test]
fn test_get_recovery_strategy_for_mismatch() {
    let validator = NonceValidator::new();

    // For mismatched nonces, should recommend AskUser strategy
    let strategy = validator.get_recovery_strategy(NonceStatus::Mismatch);
    assert_eq!(strategy, RecoveryStrategy::AskUser);
}

#[test]
fn test_get_recovery_strategy_for_valid() {
    let validator = NonceValidator::new();

    // For valid nonces, should recommend NoAction strategy
    let strategy = validator.get_recovery_strategy(NonceStatus::Valid);
    assert_eq!(strategy, RecoveryStrategy::NoAction);
}

#[test]
fn test_recovery_strategy_display() {
    // Test that recovery strategies have proper display text
    assert_eq!(RecoveryStrategy::NoAction.to_string(), "No action needed");
    assert_eq!(
        RecoveryStrategy::AskUser.to_string(),
        "User resolution required"
    );
    assert_eq!(RecoveryStrategy::SkipRecord.to_string(), "Skip this record");
    assert_eq!(RecoveryStrategy::UseLocal.to_string(), "Keep local version");
    assert_eq!(
        RecoveryStrategy::UseRemote.to_string(),
        "Use remote version"
    );
}

#[test]
fn test_nonce_status_display() {
    // Test that nonce statuses have proper display text
    assert_eq!(NonceStatus::Valid.to_string(), "Nonce is valid");
    assert_eq!(NonceStatus::Mismatch.to_string(), "Nonce mismatch detected");
}

#[test]
fn test_validator_detects_tampering_scenario() {
    let validator = NonceValidator::new();

    // Scenario: Attacker modifies encrypted data but doesn't update nonce
    let local_nonce = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let local_record = create_test_record_with_nonce(local_nonce);

    // Create sync record with tampered encrypted data
    let mut sync_record = create_sync_record_with_nonce(local_nonce);
    sync_record.encrypted_data = STANDARD.encode(b"tampered-data-12345");

    // Nonces match but this is still suspicious
    // In real scenario, decryption would fail with wrong nonce
    let result = validator.validate(&local_record, &sync_record);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), NonceStatus::Valid);
    // Note: Actual tampering detection would happen during decryption
}

#[test]
fn test_multiple_records_validation() {
    let validator = NonceValidator::new();

    // Test validating multiple records
    let records = vec![
        (
            create_test_record_with_nonce([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            create_sync_record_with_nonce([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            true,
        ),
        (
            create_test_record_with_nonce([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            create_sync_record_with_nonce([99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88]),
            false,
        ),
    ];

    for (local, sync, should_match) in records {
        let result = validator.validate(&local, &sync);
        assert!(result.is_ok());
        let status = result.unwrap();
        if should_match {
            assert_eq!(status, NonceStatus::Valid);
        } else {
            assert_eq!(status, NonceStatus::Mismatch);
        }
    }
}

// Helper functions

fn create_test_record_with_nonce(nonce: [u8; 12]) -> StoredRecord {
    StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: b"test-data".to_vec(),
        nonce,
        tags: vec!["test".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
    }
}

fn create_sync_record_with_nonce(nonce: [u8; 12]) -> SyncRecord {
    SyncRecord {
        id: Uuid::new_v4().to_string(),
        version: 1,
        record_type: RecordType::Password,
        encrypted_data: STANDARD.encode(b"test-data"),
        nonce: STANDARD.encode(nonce),
        metadata: keyring_cli::sync::export::RecordMetadata {
            name: "test".to_string(),
            tags: vec!["test".to_string()],
            platform: "test".to_string(),
            device_id: "test-device".to_string(),
        },
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
