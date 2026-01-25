//! HIBP (Have I Been Pwned) API integration for compromised password checking

use crate::crypto::record::{decrypt_payload, RecordPayload};
use crate::crypto::CryptoManager;
use crate::db::models::StoredRecord;
use crate::health::report::{HealthIssue, HealthIssueType, Severity};
use sha1::{Digest, Sha1};
use std::time::Duration;

/// Check for compromised passwords using HIBP API
pub async fn check_compromised_passwords(
    records: &[StoredRecord],
    crypto: &CryptoManager,
) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

    for record in records {
        if let Ok(password) = get_password_from_record(record, crypto) {
            if let Ok(is_compromised) = is_password_compromised(&password).await {
                if is_compromised {
                    issues.push(HealthIssue {
                        issue_type: HealthIssueType::CompromisedPassword,
                        record_names: vec![record.id.to_string()],
                        description: "Password found in data breach".to_string(),
                        severity: Severity::Critical,
                    });
                }
            }
        }
    }
    issues
}

/// Check if a password has been compromised using HIBP k-anonymity API
///
/// This function:
/// 1. Computes the SHA-1 hash of the password
/// 2. Sends only the first 5 characters (prefix) to HIBP API
/// 3. Receives a list of all password hashes with that prefix
/// 4. Checks if the remaining characters (suffix) match any entry
///
/// This method ensures the full password is never sent to the server.
pub async fn is_password_compromised(password: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use reqwest::Client;

    // Validate password length
    if password.is_empty() {
        return Ok(false);
    }

    // Compute SHA-1 hash
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    let hash_str = format!("{:X}", hash);

    // Split into prefix and suffix
    let prefix = &hash_str[0..5];
    let suffix = &hash_str[5..];

    // Query HIBP API (k-anonymity model)
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("OpenKeyring/0.1.0 (+https://github.com/open-keyring/keyring-cli)")
        .build()?;

    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HIBP API returned {}", response.status()).into());
    }

    let body = response.text().await?;

    // Check if suffix is in response
    for line in body.lines() {
        if let Some((hash_suffix, count)) = line.split_once(':') {
            if hash_suffix.to_uppercase() == suffix.to_uppercase() {
                // Password found in breach database
                let count: u64 = count.parse().unwrap_or(0);
                if count > 0 {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
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

    #[tokio::test]
    async fn test_sha1_hash() {
        let mut hasher = Sha1::new();
        hasher.update(b"password");
        let hash = hasher.finalize();
        let hash_str = format!("{:X}", hash);
        assert_eq!(hash_str.len(), 40);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_hibp_api_connection() {
        // Test that we can connect to HIBP API
        let result = is_password_compromised("password").await;
        assert!(result.is_ok(), "Should be able to connect to HIBP API");
        // "password" should be in HIBP database
        assert!(result.unwrap(), "password should be in HIBP database");
    }
}
