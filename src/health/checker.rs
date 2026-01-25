//! Main health checker orchestrating all health checks

use crate::crypto::CryptoManager;
use crate::db::models::StoredRecord;
use crate::health::{strength, hibp};
use crate::health::report::{HealthIssue, HealthIssueType, Severity};
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
                .or_insert_with(Vec::new)
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
