//! Encrypted record payload helpers

use crate::crypto::CryptoManager;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPayload {
    pub name: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

pub fn encrypt_payload(
    crypto: &CryptoManager,
    payload: &RecordPayload,
) -> Result<(Vec<u8>, [u8; 12])> {
    let bytes = serde_json::to_vec(payload)?;
    crypto.encrypt(&bytes)
}

pub fn decrypt_payload(
    crypto: &CryptoManager,
    ciphertext: &[u8],
    nonce: &[u8; 12],
) -> Result<RecordPayload> {
    let plaintext = crypto.decrypt(ciphertext, nonce)?;
    Ok(serde_json::from_slice(&plaintext)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_payload_success() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let payload = RecordPayload {
            name: "test-record".to_string(),
            username: Some("user@example.com".to_string()),
            password: "secret-password-123".to_string(),
            url: Some("https://example.com".to_string()),
            notes: Some("Test notes".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        let (ciphertext, nonce) = encrypt_payload(&crypto, &payload).unwrap();
        assert!(!ciphertext.is_empty());
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let original = RecordPayload {
            name: "roundtrip-test".to_string(),
            username: Some("testuser".to_string()),
            password: "my-secure-password".to_string(),
            url: Some("https://test.com".to_string()),
            notes: Some("Important notes".to_string()),
            tags: vec!["important".to_string(), "work".to_string()],
        };

        let (ciphertext, nonce) = encrypt_payload(&crypto, &original).unwrap();
        let decrypted = decrypt_payload(&crypto, &ciphertext, &nonce).unwrap();

        assert_eq!(decrypted.name, original.name);
        assert_eq!(decrypted.username, original.username);
        assert_eq!(decrypted.password, original.password);
        assert_eq!(decrypted.url, original.url);
        assert_eq!(decrypted.notes, original.notes);
        assert_eq!(decrypted.tags, original.tags);
    }

    #[test]
    fn test_encrypt_payload_with_empty_optional_fields() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let payload = RecordPayload {
            name: "minimal-record".to_string(),
            username: None,
            password: "password123".to_string(),
            url: None,
            notes: None,
            tags: vec![],
        };

        let (ciphertext, nonce) = encrypt_payload(&crypto, &payload).unwrap();
        let decrypted = decrypt_payload(&crypto, &ciphertext, &nonce).unwrap();

        assert_eq!(decrypted.name, "minimal-record");
        assert!(decrypted.username.is_none());
        assert_eq!(decrypted.password, "password123");
        assert!(decrypted.url.is_none());
        assert!(decrypted.notes.is_none());
        assert!(decrypted.tags.is_empty());
    }

    #[test]
    fn test_encrypt_payload_with_special_characters() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let payload = RecordPayload {
            name: "测试记录-🔒".to_string(),
            username: Some("user+test@example.com".to_string()),
            password: "p@$$w0rd!\"'\\<>|&".to_string(),
            url: Some("https://example.com?param=value&other=123".to_string()),
            notes: Some("Notes with \"quotes\" and 'apostrophes'\nNewlines too!".to_string()),
            tags: vec![
                "tag-with-dash".to_string(),
                "tag_with_underscore".to_string(),
            ],
        };

        let (ciphertext, nonce) = encrypt_payload(&crypto, &payload).unwrap();
        let decrypted = decrypt_payload(&crypto, &ciphertext, &nonce).unwrap();

        assert_eq!(decrypted.name, payload.name);
        assert_eq!(decrypted.username, payload.username);
        assert_eq!(decrypted.password, payload.password);
        assert_eq!(decrypted.url, payload.url);
        assert_eq!(decrypted.notes, payload.notes);
        assert_eq!(decrypted.tags, payload.tags);
    }

    #[test]
    fn test_encrypt_payload_different_encryptions_produce_different_ciphertexts() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let payload = RecordPayload {
            name: "same-content".to_string(),
            username: Some("user".to_string()),
            password: "password".to_string(),
            url: None,
            notes: None,
            tags: vec![],
        };

        let (ciphertext1, _) = encrypt_payload(&crypto, &payload).unwrap();
        let (ciphertext2, _) = encrypt_payload(&crypto, &payload).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn test_decrypt_payload_with_wrong_nonce_fails() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        let payload = RecordPayload {
            name: "test".to_string(),
            username: None,
            password: "password".to_string(),
            url: None,
            notes: None,
            tags: vec![],
        };

        let (ciphertext, _) = encrypt_payload(&crypto, &payload).unwrap();
        let wrong_nonce = [0u8; 12];

        let result = decrypt_payload(&crypto, &ciphertext, &wrong_nonce);
        assert!(result.is_err());
    }
}
