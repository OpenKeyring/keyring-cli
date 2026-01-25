//! Password strength checking module

use crate::crypto::CryptoManager;
use crate::db::models::StoredRecord;
use crate::health::report::{HealthIssue, HealthIssueType, Severity};
use crate::crypto::record::{decrypt_payload, RecordPayload};

/// Check for weak passwords based on strength scoring
pub fn check_weak_passwords(records: &[StoredRecord], crypto: &CryptoManager) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

    for record in records {
        if let Ok(password) = get_password_from_record(record, crypto) {
            let score = calculate_strength(&password);
            if score < 60 {
                issues.push(HealthIssue {
                    issue_type: HealthIssueType::WeakPassword,
                    record_names: vec![record.id.to_string()],
                    description: format!("Weak password (strength score: {}/100)", score),
                    severity: if score < 40 { Severity::High } else { Severity::Medium },
                });
            }
        }
    }
    issues
}

/// Calculate password strength score (0-100)
///
/// Scoring based on:
/// - Length (up to 40 points)
/// - Character variety (up to 30 points)
/// - Pattern penalties (deductions)
/// - Common password penalties (deductions)
pub fn calculate_strength(password: &str) -> u8 {
    let mut score = 0u8;

    // Length contribution (up to 40 points)
    score += (password.len().min(16) as u8 * 2).min(40);

    // Character variety (up to 30 points)
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();
    score += (variety * 8) as u8;

    // Deductions for common patterns
    if password.to_lowercase().contains("password") || password.to_lowercase().contains("qwerty") {
        score = score.saturating_sub(30);
    }

    // Check for sequential characters
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
        let is_sequential = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 { return true; }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            (curr - prev).abs() == 1
        });
        if is_sequential {
            score = score.saturating_sub(20);
            break;
        }
    }

    score.max(0).min(100)
}

/// Extract password from a stored record using decryption
fn get_password_from_record(
    record: &StoredRecord,
    crypto: &CryptoManager,
) -> Result<String, Box<dyn std::error::Error>> {
    let payload: RecordPayload = decrypt_payload(crypto, &record.encrypted_data, &record.nonce)?;
    Ok(payload.password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_weak_passwords() {
        assert!(calculate_strength("password") < 40);
        assert!(calculate_strength("123456") < 40);
        assert!(calculate_strength("qwerty") < 40);
    }

    #[test]
    fn test_weak_passwords() {
        assert!(calculate_strength("abc123") < 60);
        assert!(calculate_strength("Monkey1") < 60);
    }

    #[test]
    fn test_medium_passwords() {
        assert!(calculate_strength("MyPass123!") >= 60);
        assert!(calculate_strength("Secure-456") >= 60);
    }

    #[test]
    fn test_strong_passwords() {
        assert!(calculate_strength("MyStr0ng!P@ssw0rd#2024") >= 80);
        assert!(calculate_strength("xK9#mP2$vL5@nQ8") >= 80);
    }
}
