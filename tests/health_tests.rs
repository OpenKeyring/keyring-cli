// Health check module tests

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use keyring_cli::crypto::record::{encrypt_payload, RecordPayload};
    use keyring_cli::crypto::CryptoManager;
    use keyring_cli::db::models::StoredRecord;
    use keyring_cli::health::{HealthChecker, HealthIssueType};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_health_checker_module_exists() {
        let _records: Vec<StoredRecord> = vec![];
        // Health module structure exists - test passes if module compiles
    }

    #[tokio::test]
    async fn test_health_check_decrypts_password() {
        // Initialize crypto manager
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-master-password").unwrap();

        // Create a test record with a weak password
        let payload = RecordPayload {
            name: "weak-test".to_string(),
            username: Some("user".to_string()),
            password: "password123".to_string(), // Weak password
            url: None,
            notes: None,
            tags: vec![],
        };

        let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload).unwrap();

        let record = StoredRecord {
            id: Uuid::new_v4(),
            record_type: keyring_cli::db::models::RecordType::Password,
            encrypted_data,
            nonce,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            deleted: false,
        };

        // Run health check - should detect weak password
        // Disable leak check to avoid reqwest client issues in test environment
        let checker = HealthChecker::new(crypto).with_leaks(false);
        let issues = checker.check_all(&[record]).await;

        // Should detect at least weak password
        assert!(!issues.is_empty(), "Should detect weak password");

        // Check that weak password was detected
        let weak_found = issues
            .iter()
            .any(|i| matches!(i.issue_type, HealthIssueType::WeakPassword));
        assert!(weak_found, "Should detect weak password issue");
    }

    #[tokio::test]
    async fn test_health_check_detects_duplicates() {
        // Initialize crypto manager
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-master-password-2").unwrap();

        // Create two records with the same password
        let password = "SamePassword123!".to_string();

        let record1 = create_record("account-1", &password, &crypto);
        let record2 = create_record("account-2", &password, &crypto);

        // Run health check - should detect duplicate passwords
        let checker = HealthChecker::new(crypto).with_leaks(false); // Disable leak check for offline test
        let issues = checker.check_all(&[record1, record2]).await;

        // Check that duplicate password was detected
        let dup_found = issues
            .iter()
            .any(|i| matches!(i.issue_type, HealthIssueType::DuplicatePassword));
        assert!(dup_found, "Should detect duplicate password issue");
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
            record_type: keyring_cli::db::models::RecordType::Password,
            encrypted_data,
            nonce,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: 1,
            deleted: false,
        }
    }
}
