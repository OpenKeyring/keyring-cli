// Health check integration tests

use chrono::Utc;
use keyring_cli::crypto::record::{encrypt_payload, RecordPayload};
use keyring_cli::crypto::CryptoManager;
use keyring_cli::db::models::{RecordType, StoredRecord};
use keyring_cli::health::{HealthChecker, HealthIssueType, HealthReport};
use uuid::Uuid;

#[tokio::test]
async fn test_full_health_check_workflow() {
    // Initialize crypto
    let mut crypto = CryptoManager::new();
    crypto.initialize("test-password").unwrap();

    // Create test records with various health issues
    let records = create_test_records(&crypto);

    // Run health checks
    let checker = HealthChecker::new(crypto).with_leaks(false); // Disable leak check for offline test
    let issues = checker.check_all(&records).await;
    let report = HealthReport::from_issues(records.len(), issues);

    // Verify results
    assert!(!report.is_healthy(), "Should find health issues");
    assert!(
        report.weak_password_count > 0,
        "Should detect weak passwords"
    );
    assert!(
        report.duplicate_password_count > 0,
        "Should detect duplicates"
    );
}

#[tokio::test]
async fn test_health_check_with_only_strong_passwords() {
    // Initialize crypto
    let mut crypto = CryptoManager::new();
    crypto.initialize("strong-test-password").unwrap();

    // Create records with only strong, unique passwords
    let records = vec![
        create_record("strong-1", "aB3$xK9#mP2$vL5@nQ8!", &crypto),
        create_record("strong-2", "MyStr0ng!P@ssw0rd#2024", &crypto),
        create_record("strong-3", "CorrectHorse!Battery#Staple2024", &crypto),
    ];

    // Run health checks
    let checker = HealthChecker::new(crypto).with_leaks(false);
    let issues = checker.check_all(&records).await;
    let report = HealthReport::from_issues(records.len(), issues);

    // Should be healthy (no weak passwords or duplicates)
    assert_eq!(
        report.weak_password_count, 0,
        "Should have no weak passwords"
    );
    assert_eq!(
        report.duplicate_password_count, 0,
        "Should have no duplicates"
    );
}

#[tokio::test]
async fn test_duplicate_detection_across_many_records() {
    // Initialize crypto
    let mut crypto = CryptoManager::new();
    crypto.initialize("dup-test-password").unwrap();

    // Create many records with some duplicates
    let records = vec![
        create_record("unique-1", "Unique!Pass#123", &crypto),
        create_record("shared-1", "Shared!Pass#456", &crypto),
        create_record("shared-2", "Shared!Pass#456", &crypto), // Same as shared-1
        create_record("shared-3", "Shared!Pass#456", &crypto), // Same as shared-1 and shared-2
        create_record("unique-2", "Another!Unique#789", &crypto),
    ];

    // Run health checks (only duplicates)
    let checker = HealthChecker::new(crypto)
        .with_weak(false)
        .with_duplicates(true)
        .with_leaks(false);
    let issues = checker.check_all(&records).await;

    // Should find exactly one duplicate issue covering 3 records
    assert_eq!(issues.len(), 1, "Should find exactly one duplicate issue");
    assert_eq!(
        issues[0].record_names.len(),
        3,
        "Duplicate should involve 3 records"
    );
    assert!(matches!(
        issues[0].issue_type,
        HealthIssueType::DuplicatePassword
    ));
}

#[tokio::test]
async fn test_weak_password_severity_levels() {
    // Initialize crypto
    let mut crypto = CryptoManager::new();
    crypto.initialize("severity-test-password").unwrap();

    // Create records with different weakness levels
    let records = vec![
        create_record("very-weak", "password", &crypto), // Very weak (< 40)
        create_record("somewhat-weak", "Monkey1", &crypto), // Somewhat weak (40-60)
    ];

    // Run health checks
    let checker = HealthChecker::new(crypto)
        .with_weak(true)
        .with_duplicates(false)
        .with_leaks(false);
    let issues = checker.check_all(&records).await;

    // Should find both weak passwords
    assert_eq!(
        issues.len(),
        2,
        "Should find both weak passwords: {:?}",
        issues
    );

    // Both should be WeakPassword type
    assert!(
        issues
            .iter()
            .all(|i| matches!(i.issue_type, HealthIssueType::WeakPassword)),
        "All issues should be WeakPassword type"
    );

    // At least one should have High severity (very weak)
    assert!(
        issues
            .iter()
            .any(|i| i.severity >= keyring_cli::health::report::Severity::High),
        "At least one password should have High severity"
    );
}

fn create_test_records(crypto: &CryptoManager) -> Vec<StoredRecord> {
    // Create test records with various issues
    vec![
        create_record("weak-password-1", "password123", crypto),
        create_record("weak-password-2", "qwerty", crypto),
        create_record("duplicate-1", "same-password!", crypto),
        create_record("duplicate-2", "same-password!", crypto),
    ]
}

fn create_record(name: &str, password: &str, crypto: &CryptoManager) -> StoredRecord {
    let payload = RecordPayload {
        name: name.to_string(),
        username: None,
        password: password.to_string(),
        url: None,
        notes: None,
        tags: vec![],
    };

    let (encrypted_data, nonce) = encrypt_payload(crypto, &payload).unwrap();

    StoredRecord {
        id: Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
        deleted: false,
    }
}
