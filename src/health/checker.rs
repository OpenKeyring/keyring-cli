//! Main health checker orchestrating all health checks

use crate::crypto::CryptoManager;
use crate::db::models::StoredRecord;
use crate::health::report::{HealthIssue, HealthIssueType, Severity};
use crate::health::{hibp, strength};
use std::collections::HashMap;

/// Main health checker that orchestrates all health checks
pub struct HealthChecker {
    check_weak: bool,
    check_duplicates: bool,
    check_leaks: bool,
    crypto: CryptoManager,
}

impl HealthChecker {
    /// Create a new health checker with the given crypto manager
    pub fn new(crypto: CryptoManager) -> Self {
        Self {
            check_weak: true,
            check_duplicates: true,
            check_leaks: true,
            crypto,
        }
    }

    /// Configure whether to check for weak passwords
    pub fn with_weak(mut self, enabled: bool) -> Self {
        self.check_weak = enabled;
        self
    }

    /// Configure whether to check for duplicate passwords
    pub fn with_duplicates(mut self, enabled: bool) -> Self {
        self.check_duplicates = enabled;
        self
    }

    /// Configure whether to check for leaked/compromised passwords
    pub fn with_leaks(mut self, enabled: bool) -> Self {
        self.check_leaks = enabled;
        self
    }

    /// Run all enabled health checks on the given records
    pub async fn check_all(&self, records: &[StoredRecord]) -> Vec<HealthIssue> {
        let mut issues = Vec::new();

        if self.check_weak {
            issues.extend(strength::check_weak_passwords(records, &self.crypto));
        }

        if self.check_duplicates {
            issues.extend(check_duplicates(records, &self.crypto));
        }

        if self.check_leaks {
            issues.extend(hibp::check_compromised_passwords(records, &self.crypto).await);
        }

        issues
    }
}

/// Check for duplicate passwords across records
fn check_duplicates(records: &[StoredRecord], crypto: &CryptoManager) -> Vec<HealthIssue> {
    let mut password_counts: HashMap<String, Vec<String>> = HashMap::new();

    for record in records {
        if let Ok(password) = get_password_from_record(record, crypto) {
            password_counts
                .entry(password.clone())
                .or_default()
                .push(record.id.to_string());
        }
    }

    let mut issues = Vec::new();
    for (_password, ids) in password_counts {
        if ids.len() > 1 {
            issues.push(HealthIssue {
                issue_type: HealthIssueType::DuplicatePassword,
                record_names: ids.clone(),
                description: format!("Password used by {} accounts", ids.len()),
                severity: Severity::Medium,
            });
        }
    }
    issues
}

/// Extract password from a stored record using decryption
fn get_password_from_record(
    record: &StoredRecord,
    crypto: &CryptoManager,
) -> Result<String, Box<dyn std::error::Error>> {
    use crate::crypto::record::{decrypt_payload, RecordPayload};

    let payload: RecordPayload = decrypt_payload(crypto, &record.encrypted_data, &record.nonce)?;
    Ok(payload.password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::record::{encrypt_payload, RecordPayload};
    use chrono::Utc;
    use uuid::Uuid;

    fn setup_test_crypto() -> CryptoManager {
        use crate::crypto::argon2id::derive_key;

        let password = "test-password";
        let salt = [0u8; 16];
        let master_key_vec = derive_key(password, &salt).unwrap();
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&master_key_vec[..32]);

        let mut crypto = CryptoManager::new();
        crypto.initialize_with_key(key_array);
        crypto
    }

    fn create_test_record(id: &str, password: &str, crypto: &CryptoManager) -> StoredRecord {
        let payload = RecordPayload {
            name: "test_record".to_string(),
            password: password.to_string(),
            username: Some("testuser".to_string()),
            url: Some("https://example.com".to_string()),
            notes: None,
            tags: vec![],
        };

        let (encrypted_data, nonce) = encrypt_payload(crypto, &payload).unwrap();

        StoredRecord {
            id: Uuid::parse_str(id).unwrap(),
            record_type: crate::db::models::RecordType::Password,
            encrypted_data,
            nonce,
            tags: vec![],
            group_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            deleted: false,
        }
    }

    // HealthChecker creation tests
    #[test]
    fn test_health_checker_new() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto);

        assert!(checker.check_weak);
        assert!(checker.check_duplicates);
        assert!(checker.check_leaks);
    }

    #[test]
    fn test_health_checker_with_weak_disabled() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto).with_weak(false);

        assert!(!checker.check_weak);
        assert!(checker.check_duplicates);
        assert!(checker.check_leaks);
    }

    #[test]
    fn test_health_checker_with_duplicates_disabled() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto).with_duplicates(false);

        assert!(checker.check_weak);
        assert!(!checker.check_duplicates);
        assert!(checker.check_leaks);
    }

    #[test]
    fn test_health_checker_with_leaks_disabled() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto).with_leaks(false);

        assert!(checker.check_weak);
        assert!(checker.check_duplicates);
        assert!(!checker.check_leaks);
    }

    #[test]
    fn test_health_checker_with_all_disabled() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_duplicates(false)
            .with_leaks(false);

        assert!(!checker.check_weak);
        assert!(!checker.check_duplicates);
        assert!(!checker.check_leaks);
    }

    // check_all tests
    #[tokio::test]
    async fn test_check_all_empty_records() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto);
        let records: Vec<StoredRecord> = vec![];

        let issues = checker.check_all(&records).await;

        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn test_check_all_with_weak_password() {
        let crypto = setup_test_crypto();
        let record1 =
            create_test_record("550e8400-e29b-41d4-a716-446655440001", "password", &crypto);
        let record2 = create_test_record(
            "550e8400-e29b-41d4-a716-446655440002",
            "StrongP@ss123!",
            &crypto,
        );

        let checker = HealthChecker::new(crypto)
            .with_duplicates(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record1, record2]).await;

        // Only "password" should be flagged as weak
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, HealthIssueType::WeakPassword);
    }

    #[tokio::test]
    async fn test_check_all_with_duplicate_passwords() {
        let crypto = setup_test_crypto();
        let record1 = create_test_record(
            "550e8400-e29b-41d4-a716-446655440001",
            "SamePass123!",
            &crypto,
        );
        let record2 = create_test_record(
            "550e8400-e29b-41d4-a716-446655440002",
            "SamePass123!",
            &crypto,
        );
        let record3 = create_test_record(
            "550e8400-e29b-41d4-a716-446655440003",
            "DifferentPass!",
            &crypto,
        );

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record1, record2, record3]).await;

        // Should find one duplicate issue with 2 record IDs
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].issue_type, HealthIssueType::DuplicatePassword);
        assert_eq!(issues[0].record_names.len(), 2);
    }

    #[tokio::test]
    async fn test_check_all_with_multiple_duplicate_groups() {
        let crypto = setup_test_crypto();
        let record1 = create_test_record("550e8400-e29b-41d4-a716-446655440001", "Pass1!", &crypto);
        let record2 = create_test_record("550e8400-e29b-41d4-a716-446655440002", "Pass1!", &crypto);
        let record3 = create_test_record("550e8400-e29b-41d4-a716-446655440003", "Pass2!", &crypto);
        let record4 = create_test_record("550e8400-e29b-41d4-a716-446655440004", "Pass2!", &crypto);
        let record5 = create_test_record("550e8400-e29b-41d4-a716-446655440005", "Pass2!", &crypto);

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_leaks(false);
        let issues = checker
            .check_all(&[record1, record2, record3, record4, record5])
            .await;

        // Should find two duplicate issues
        assert_eq!(issues.len(), 2);
        // Both groups should have records
        let group1_count = issues[0].record_names.len();
        let group2_count = issues[1].record_names.len();
        // One group has 2 records, the other has 3 (order not guaranteed)
        assert!(
            (group1_count == 2 && group2_count == 3) || (group1_count == 3 && group2_count == 2)
        );
    }

    #[tokio::test]
    async fn test_check_all_no_duplicates_unique_passwords() {
        let crypto = setup_test_crypto();
        let record1 = create_test_record("550e8400-e29b-41d4-a716-446655440001", "Pass1!", &crypto);
        let record2 = create_test_record("550e8400-e29b-41d4-a716-446655440002", "Pass2!", &crypto);
        let record3 = create_test_record("550e8400-e29b-41d4-a716-446655440003", "Pass3!", &crypto);

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record1, record2, record3]).await;

        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn test_check_all_disabled_all_checks() {
        let crypto = setup_test_crypto();
        let record1 =
            create_test_record("550e8400-e29b-41d4-a716-446655440001", "password", &crypto);
        let record2 =
            create_test_record("550e8400-e29b-41d4-a716-446655440002", "password", &crypto);

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_duplicates(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record1, record2]).await;

        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn test_check_all_duplicate_severity() {
        let crypto = setup_test_crypto();
        let record1 =
            create_test_record("550e8400-e29b-41d4-a716-446655440001", "SamePass!", &crypto);
        let record2 =
            create_test_record("550e8400-e29b-41d4-a716-446655440002", "SamePass!", &crypto);

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record1, record2]).await;

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Medium);
        assert!(issues[0].description.contains("2 accounts"));
    }

    // Builder pattern chaining tests
    #[test]
    fn test_health_checker_builder_chaining() {
        let crypto = setup_test_crypto();
        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_duplicates(true)
            .with_leaks(false);

        assert!(!checker.check_weak);
        assert!(checker.check_duplicates);
        assert!(!checker.check_leaks);
    }

    // Edge case tests
    #[tokio::test]
    async fn test_check_all_with_decryption_failure() {
        let crypto = setup_test_crypto();
        // Create a record that will fail to decrypt (using different key derivation)
        let mut other_crypto = CryptoManager::new();
        use crate::crypto::argon2id::derive_key;
        let other_salt = [1u8; 16]; // Different salt
        let other_key_vec = derive_key("different-password", &other_salt).unwrap();
        let mut other_key_array = [0u8; 32];
        other_key_array.copy_from_slice(&other_key_vec[..32]);
        other_crypto.initialize_with_key(other_key_array);

        let record = create_test_record(
            "550e8400-e29b-41d4-a716-446655440001",
            "test",
            &other_crypto,
        );

        let checker = HealthChecker::new(crypto)
            .with_weak(false)
            .with_leaks(false);
        let issues = checker.check_all(&[record]).await;

        // Decryption failure should be handled gracefully (no issues)
        assert!(issues.is_empty());
    }
}
