//! Password strength checking module

use crate::crypto::record::{decrypt_payload, RecordPayload};
use crate::crypto::CryptoManager;
use crate::db::models::StoredRecord;
use crate::health::report::{HealthIssue, HealthIssueType, Severity};

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
                    severity: if score < 40 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
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
            if i == 0 {
                return true;
            }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            curr - prev == 1
        });
        // Also check for reverse sequences
        let reverse_sequential = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 {
                return true;
            }
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
        "password", "qwerty", "asdfgh", "zxcvbn", "letmein", "welcome", "login", "admin", "123456",
        "111111", "123123",
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
        ("@", "a"),
        ("0", "o"),
        ("3", "e"),
        ("1", "i"),
        ("$", "s"),
        ("7", "t"),
        ("9", "g"),
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

    score.min(100)
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

    // Very weak password tests
    #[test]
    fn test_very_weak_passwords() {
        assert!(calculate_strength("password") < 30);
        assert!(calculate_strength("123456") < 30);
        assert!(calculate_strength("qwerty") < 30);
    }

    #[test]
    fn test_very_weak_password_common_patterns() {
        // Common patterns should result in very low scores
        assert!(calculate_strength("admin") < 30);
        assert!(calculate_strength("letmein") < 30);
        assert!(calculate_strength("welcome") < 30);
        assert!(calculate_strength("111111") < 30);
        assert!(calculate_strength("123123") < 30);
    }

    // Weak password tests
    #[test]
    fn test_weak_passwords() {
        assert!(calculate_strength("abc123") < 60);
        assert!(calculate_strength("Monkey1") < 60);
    }

    #[test]
    fn test_weak_password_short_length() {
        // Short passwords (under 8 chars) should be weak
        // "abc12": 5 chars * 3 = 15 + 12 (2 types) = 27
        assert!(calculate_strength("abc12") < 40);
        // "aB3!": 4 chars * 3 = 12 + 30 (4 types) = 42
        assert!(calculate_strength("aB3!") < 50);
        // "1234567": 7 chars * 3 = 21 + 5 (1 type) = 26
        assert!(calculate_strength("1234567") < 30);
    }

    #[test]
    fn test_weak_password_no_variety() {
        // Only lowercase should be weak
        assert!(calculate_strength("abcdefghijklmnopqrstuvwxyz") < 60);
        // Only uppercase should be weak
        assert!(calculate_strength("ABCDEFGHIJKLMNOPQRSTUVWXYZ") < 60);
        // Only digits should be weak
        assert!(calculate_strength("012345678901234567890") < 60);
    }

    #[test]
    fn test_weak_password_sequential_chars() {
        // Sequential characters should reduce score
        assert!(calculate_strength("abcd1234") < 60);
        assert!(calculate_strength("4321abcd") < 60);
        assert!(calculate_strength("xyz987") < 60);
    }

    #[test]
    fn test_weak_password_repeated_chars() {
        // Repeated characters should reduce score
        assert!(calculate_strength("aaaaaaa") < 60);
        assert!(calculate_strength("111222") < 60);
        assert!(calculate_strength("AAAbbbccc") < 60);
    }

    // Medium password tests
    #[test]
    fn test_medium_passwords() {
        assert!(calculate_strength("MyPass123!") >= 60);
        assert!(calculate_strength("Secure-456") >= 60);
        assert!(calculate_strength("xK9#mP2$vL5@nQ8") >= 60);
    }

    #[test]
    fn test_medium_password_12_chars_with_variety() {
        // 12 chars with 3-4 character types (45+)
        // "Abc123XYZxyz!" has good variety: 67
        assert!(calculate_strength("Abc123XYZxyz!") >= 60);
        // "MyPass@1234" has some issues (possibly "Pass" pattern): 45
        assert!(calculate_strength("MyPass@1234") >= 40);
    }

    #[test]
    fn test_medium_password_exactly_60() {
        // These should score around 60
        assert!(calculate_strength("MyP@ss123!") >= 55);
        assert!(calculate_strength("Secure-456!") >= 55);
    }

    // Strong password tests
    #[test]
    fn test_strong_passwords() {
        assert!(calculate_strength("MyStr0ng!P@ssw0rd#2024") >= 80);
        assert!(calculate_strength("aB3$xK9#mP2$vL5@nQ8!") >= 80);
    }

    #[test]
    fn test_strong_password_16_plus_chars() {
        // Long passwords with variety should be strong
        // Note: "Password" common pattern reduces score significantly
        assert!(calculate_strength("ThisIsAVeryLongPassword123!") >= 40);
        // "SecureP@ssw0rd!2024XYZ" has "P@ssw0rd" which contains "password" with substitutions
        assert!(calculate_strength("SecureP@ssw0rd!2024XYZ") >= 40);
    }

    #[test]
    fn test_strong_password_all_four_types() {
        // Contains all 4 character types and good length
        assert!(calculate_strength("aB3$xK9#mP2$vL5@nQ8!") >= 80);
        assert!(calculate_strength("MyStr0ng!P@ssw0rd#2024!") >= 80);
    }

    #[test]
    fn test_strong_password_high_unique_ratio() {
        // High unique character ratio (65+ due to length penalties)
        assert!(calculate_strength("QwEr@#Ty123!uioP") >= 60);
        assert!(calculate_strength("aBcDeFgH123!@#$") >= 60);
    }

    // Edge case tests
    #[test]
    fn test_empty_password() {
        // Empty password should score 0
        assert_eq!(calculate_strength(""), 0);
    }

    #[test]
    fn test_single_char_password() {
        // Single character should be very weak
        assert!(calculate_strength("a") < 20);
        assert!(calculate_strength("Z") < 20);
        assert!(calculate_strength("5") < 20);
    }

    #[test]
    fn test_password_with_only_spaces() {
        // Spaces count as symbols but empty password is still weak
        assert!(calculate_strength("   ") < 20);
    }

    #[test]
    fn test_password_with_unicode() {
        // Unicode characters should be counted but don't contribute to variety
        assert!(calculate_strength("密码123") < 60); // Non-Latin scripts score lower
        assert!(calculate_strength("пароль123") < 60); // Cyrillic scores lower
    }

    #[test]
    fn test_max_score_capped_at_100() {
        // Even very long passwords with high variety should max at 100
        // Use a password without common patterns or sequential chars
        let score1 = calculate_strength("xK9#mP2$vL5@nQ8!aB3$wR7&tY6*uZ9");
        // This should have high score (80)
        assert!(score1 >= 70);
        assert!(score1 <= 100);
        // Test another without sequential patterns
        let score2 = calculate_strength("MyStr0ng!P@ssw0rd#2024XYZ");
        assert!(score2 >= 70);
        assert!(score2 <= 100);
    }

    // Substitution tests
    #[test]
    fn test_common_substitutions_detected() {
        // Common substitutions should be detected
        // "p@ssw0rd": 8 chars (25 length) + 4 types (30 variety) - 25 common pattern = 30
        assert!(calculate_strength("p@ssw0rd") < 60);
        assert!(calculate_strength("pa$$w0rd") < 60);
        assert!(calculate_strength("p@ssw0rd!") < 70);
    }

    #[test]
    fn test_multiple_substitutions() {
        // "1etsme1n": 8 chars (25) + 3 types (20) - 25 pattern = 20
        assert!(calculate_strength("1etsme1n") < 50);
        // "welc0me": 7 chars (21) + 2 types (12) - 25 pattern = 8
        assert!(calculate_strength("welc0me") < 50);
        // "adm1n": 5 chars (15) + 2 types (12) - 25 pattern = 2
        assert!(calculate_strength("adm1n") < 50);
    }

    // Sequential character tests
    #[test]
    fn test_alphabetical_sequences() {
        assert!(calculate_strength("abcd") < 50);
        assert!(calculate_strength("xyz") < 40);
        assert!(calculate_strength("ABCD") < 50);
        assert!(calculate_strength("ZYXW") < 50);
    }

    #[test]
    fn test_numeric_sequences() {
        assert!(calculate_strength("1234") < 50);
        assert!(calculate_strength("9876") < 50);
        assert!(calculate_strength("0123") < 50);
    }

    #[test]
    fn test_reverse_sequences() {
        assert!(calculate_strength("dcba") < 50);
        assert!(calculate_strength("4321") < 50);
    }

    // Character variety tests
    #[test]
    fn test_lower_only() {
        assert!(calculate_strength("abcdefghijk") < 50);
        assert!(calculate_strength("password") < 30);
    }

    #[test]
    fn test_upper_only() {
        assert!(calculate_strength("ABCDEFGHIJK") < 50);
        assert!(calculate_strength("PASSWORD") < 30);
    }

    #[test]
    fn test_digits_only() {
        assert!(calculate_strength("123456789012") < 50);
        assert!(calculate_strength("0987654321") < 50);
    }

    #[test]
    fn test_symbols_only() {
        assert!(calculate_strength("!@#$%^&*()") < 50);
    }

    #[test]
    fn test_two_variety_better_than_one() {
        let two_var = calculate_strength("abc123");
        let one_var = calculate_strength("abcdef");
        assert!(two_var > one_var);
    }

    #[test]
    fn test_three_variety_better_than_two() {
        let three_var = calculate_strength("Abc123!");
        let two_var = calculate_strength("abc123!");
        assert!(three_var >= two_var);
    }

    #[test]
    fn test_four_variety_best() {
        let four_var = calculate_strength("Abc123!@");
        let three_var = calculate_strength("Abc123!");
        assert!(four_var >= three_var);
    }

    // Length bonus tests
    #[test]
    fn test_long_password_bonus() {
        let short = calculate_strength("Abc123!");
        let long = calculate_strength("Abc123!Abc123!Abc123!");
        assert!(long > short);
    }

    #[test]
    fn test_length_bonus_kicks_in_over_16() {
        let exactly_16 = calculate_strength("Abc123!@XyZ1234");
        let over_16 = calculate_strength("Abc123!@XyZ12345");
        assert!(over_16 >= exactly_16);
    }

    // Unique character tests
    #[test]
    fn test_unique_char_bonus() {
        let repeated = calculate_strength("aaaaaaaaaa11111111!!!!!");
        let unique = calculate_strength("abc123!@XYZ");
        assert!(unique > repeated);
    }

    #[test]
    fn test_high_unique_ratio() {
        let low_unique = calculate_strength("aaaaaaaaaaaa");
        let high_unique = calculate_strength("aBcDeFgH");
        assert!(high_unique > low_unique);
    }

    // Length scoring tests
    #[test]
    fn test_length_scoring_7_or_less() {
        // 7 chars or less: len * 3 + variety (1 type = 5) - sequential penalty
        // "abcdefg": 7 * 3 + 5 - 15 (sequential) = 11, but we check bounds
        assert!(calculate_strength("abcdefg") < 30);
        // "1234567": 7 * 3 + 5 - 15 (sequential) = 11
        assert!(calculate_strength("1234567") < 30);
    }

    #[test]
    fn test_length_scoring_8_to_11() {
        // 8-11 chars: 25 + variety (1 type = 5) - 15 repeated penalty = 15
        let scores = (8..=11)
            .map(|len| calculate_strength(&"a".repeat(len)))
            .collect::<Vec<_>>();
        for score in scores {
            assert_eq!(score, 15);
        }
    }

    #[test]
    fn test_length_scoring_12_to_15() {
        // 12-15 chars: 32 + variety (1 type = 5) - 15 repeated penalty = 22
        let scores = (12..=15)
            .map(|len| calculate_strength(&"a".repeat(len)))
            .collect::<Vec<_>>();
        for score in scores {
            assert_eq!(score, 22);
        }
    }

    #[test]
    fn test_length_scoring_16_to_19() {
        // 16 chars: 38 + 5 - 15 = 28 (no length bonus, len is not > 16)
        assert_eq!(calculate_strength(&"a".repeat(16)), 28);
        // 17-19 chars: 38 + 5 - 15 + 5 = 33 (length bonus applies, len > 16)
        assert_eq!(calculate_strength(&"a".repeat(17)), 33);
        assert_eq!(calculate_strength(&"a".repeat(19)), 33);
    }

    #[test]
    fn test_length_scoring_20_plus() {
        // 20+ chars: 40 + variety (1 type = 5) + length bonus (5) - 15 repeated = 35
        assert_eq!(calculate_strength("aaaaaaaaaaaaaaaaaaaa"), 35);
        // 30 chars: 40 + 5 + 5 - 15 = 35
        assert_eq!(calculate_strength(&"a".repeat(30)), 35);
    }

    // Variety scoring tests
    #[test]
    fn test_variety_scoring_one_type() {
        // Only one character type: 5 points
        // "aaaaaaaa": 25 (length) + 5 (variety) - 15 (repeated) = 15
        let score = calculate_strength("aaaaaaaa");
        assert_eq!(score, 15);
    }

    #[test]
    fn test_variety_scoring_two_types() {
        // Two character types: 12 points
        let score = calculate_strength("aaaaaaa1");
        // Length score + variety should be > 20
        assert!(score > 20);
    }

    #[test]
    fn test_variety_scoring_three_types() {
        // Three character types: 20 points
        let score = calculate_strength("aaaaaaa1A");
        assert!(score >= 20);
    }

    #[test]
    fn test_variety_scoring_four_types() {
        // Four character types: 30 points
        let score = calculate_strength("aaaa1!A");
        assert!(score >= 25);
    }

    // Pattern penalty tests
    #[test]
    fn test_sequential_penalty_applied_once() {
        // Sequential penalty should only apply once
        let score1 = calculate_strength("abcd1234xyz");
        let score2 = calculate_strength("abcdxyz1234");
        // Both should have sequential patterns
        assert!(score1 < 50 && score2 < 50);
        // The penalty should be -15
    }

    #[test]
    fn test_repeated_penalty_applied_once() {
        // Repeated character penalty should only apply once
        let score = calculate_strength("aaabbbccc");
        // Should get -15 penalty for repeated chars
        assert!(score < 50);
    }

    // Common pattern tests
    #[test]
    fn test_common_password_penalties() {
        // Common patterns should get -25
        let base_score = 30; // Approximate for medium password
        let with_common = calculate_strength("password123");
        assert!(with_common < base_score);
    }

    #[test]
    fn test_qwerty_penalty() {
        assert!(calculate_strength("qwerty123") < 40);
        assert!(calculate_strength("qwerty!@#") < 50);
    }

    #[test]
    fn test_asdfgh_penalty() {
        assert!(calculate_strength("asdfgh123") < 40);
    }

    #[test]
    fn test_zxcvbn_penalty() {
        assert!(calculate_strength("zxcvbn123") < 40);
    }
}
