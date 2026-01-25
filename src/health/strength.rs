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
    let mut reasons = Vec::new();

    // 1. Length scoring (up to 40 points)
    let length_score = match password.len() {
        0..=7 => (password.len() * 3) as u8,
        8..=11 => 25,
        12..=15 => 32,
        16..=19 => 38,
        _ => 40,
    };
    score += length_score;

    if password.len() < 12 {
        reasons.push("Too short (use 12+ characters)".to_string());
    }

    // 2. Character variety (up to 30 points)
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety_count = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();

    let variety_score = match variety_count {
        1 => 5,
        2 => 12,
        3 => 20,
        4 => 30,
        _ => 0,
    };
    score += variety_score;

    if variety_count < 3 {
        reasons.push("Lacks character variety".to_string());
    }

    // 3. Pattern penalties
    // Check for sequential characters (keyboard or alphabet)
    // Only apply this penalty if the sequence is 4+ characters
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(4) {
        // Check for true sequences (each char is exactly +1 from previous)
        let sequential = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 { return true; }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            curr - prev == 1
        });
        // Also check for reverse sequences
        let reverse_sequential = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 { return true; }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            prev - curr == 1
        });
        if sequential || reverse_sequential {
            score = score.saturating_sub(15);
            reasons.push("Contains sequential characters".to_string());
            break;
        }
    }

    // Check for repeated characters (3+ in a row)
    for window in chars.windows(3) {
        if window.iter().all(|&c| c == window[0]) {
            score = score.saturating_sub(15);
            reasons.push("Contains repeated characters".to_string());
            break;
        }
    }

    // 4. Common pattern penalties
    let password_lower = password.to_lowercase();

    let common_patterns = [
        "password", "qwerty", "asdfgh", "zxcvbn",
        "letmein", "welcome", "login", "admin",
        "123456", "111111", "123123",
    ];

    for pattern in &common_patterns {
        if password_lower.contains(pattern) {
            score = score.saturating_sub(25);
            reasons.push(format!("Contains common pattern: {}", pattern));
            break;
        }
    }

    // Check for common substitutions (e.g., p@ssw0rd)
    let substitutions = [
        ("@", "a"), ("0", "o"), ("3", "e"), ("1", "i"),
        ("$", "s"), ("7", "t"), ("9", "g"),
    ];

    let subbed = password_lower.clone();
    for (sub, orig) in &substitutions {
        if subbed.contains(orig) {
            let subbed_with = subbed.replace(sub, orig);
            if common_patterns.iter().any(|p| subbed_with.contains(p)) {
                score = score.saturating_sub(15);
                reasons.push("Uses common character substitutions".to_string());
                break;
            }
        }
    }

    // 5. Bonus for length > 16
    if password.len() > 16 {
        score += 5;
    }

    // 6. Bonus for unique characters
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    if unique_chars.len() as f64 / password.len() as f64 > 0.7 {
        score += 5;
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
        assert!(calculate_strength("password") < 30);
        assert!(calculate_strength("123456") < 30);
        assert!(calculate_strength("qwerty") < 30);
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
        // 14-char password with 4 types should be medium
        assert!(calculate_strength("xK9#mP2$vL5@nQ8") >= 60);
    }

    #[test]
    fn test_strong_passwords() {
        // Long password (16+ chars) with 4 types should be strong
        assert!(calculate_strength("MyStr0ng!P@ssw0rd#2024") >= 80);
        // Test with a simpler known-strong password
        assert!(calculate_strength("aB3$xK9#mP2$vL5@nQ8!") >= 80);
    }
}
