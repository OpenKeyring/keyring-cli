//! Keystore for key hierarchy persistence

use crate::crypto::{argon2id, bip39, keywrap};
use crate::error::{KeyringError, Result};
use crate::types::SensitiveString;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const KEYSTORE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct KeyStoreFile {
    version: u32,
    master_salt: String,
    master_hash: String,
    wrapped_dek: String,
    wrapped_dek_nonce: String,
    wrapped_device_key: String,
    wrapped_device_key_nonce: String,
    recovery_key_hash: String,
    created_at: i64,
}

#[derive(Debug)]
pub struct KeyStore {
    pub dek: SensitiveString<Vec<u8>>,
    pub device_key: [u8; 32],
    pub recovery_key: Option<String>,
}

impl KeyStore {
    /// Get a reference to the DEK as a byte slice
    pub fn get_dek(&self) -> &[u8] {
        self.dek.get().as_slice()
    }

    pub fn initialize(path: &Path, master_password: &str) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let master_salt = argon2id::generate_salt();
        let master_key = derive_master_key(master_password, &master_salt)?;
        let master_hash = STANDARD.encode(master_key);

        let dek = generate_random_key();
        let device_key = generate_random_key();

        let (wrapped_dek, wrapped_dek_nonce) = keywrap::wrap_key(&dek, &master_key)?;
        let (wrapped_device_key, wrapped_device_key_nonce) =
            keywrap::wrap_key(&device_key, &master_key)?;

        let recovery_key = bip39::generate_mnemonic(24).map_err(|e| KeyringError::Crypto {
            context: e.to_string(),
        })?;
        let recovery_key_hash = hash_recovery_key(&recovery_key);

        let keystore_file = KeyStoreFile {
            version: KEYSTORE_VERSION,
            master_salt: STANDARD.encode(master_salt),
            master_hash,
            wrapped_dek: STANDARD.encode(wrapped_dek),
            wrapped_dek_nonce: STANDARD.encode(wrapped_dek_nonce),
            wrapped_device_key: STANDARD.encode(wrapped_device_key),
            wrapped_device_key_nonce: STANDARD.encode(wrapped_device_key_nonce),
            recovery_key_hash,
            created_at: chrono::Utc::now().timestamp(),
        };

        let content = serde_json::to_string_pretty(&keystore_file)?;
        fs::write(path, content)?;

        Ok(Self {
            dek: SensitiveString::new(dek.to_vec()),
            device_key,
            recovery_key: Some(recovery_key),
        })
    }

    pub fn unlock(path: &Path, master_password: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let keystore_file: KeyStoreFile = serde_json::from_str(&content)?;

        if keystore_file.version != KEYSTORE_VERSION {
            return Err(KeyringError::InvalidInput {
                context: format!("Unsupported keystore version: {}", keystore_file.version),
            });
        }

        let master_salt = decode_fixed_salt(&keystore_file.master_salt)?;
        let master_key = derive_master_key(master_password, &master_salt)?;
        let derived_hash = STANDARD.encode(master_key);

        if derived_hash != keystore_file.master_hash {
            return Err(KeyringError::AuthenticationFailed {
                reason: "Master password verification failed".to_string(),
            });
        }

        let wrapped_dek =
            STANDARD
                .decode(keystore_file.wrapped_dek)
                .map_err(|e| KeyringError::Crypto {
                    context: format!("Invalid wrapped DEK encoding: {}", e),
                })?;
        let wrapped_dek_nonce = decode_fixed_nonce(&keystore_file.wrapped_dek_nonce)?;
        let dek = keywrap::unwrap_key(&wrapped_dek, &wrapped_dek_nonce, &master_key)?;

        let wrapped_device_key =
            STANDARD
                .decode(keystore_file.wrapped_device_key)
                .map_err(|e| KeyringError::Crypto {
                    context: format!("Invalid wrapped device key encoding: {}", e),
                })?;
        let wrapped_device_key_nonce = decode_fixed_nonce(&keystore_file.wrapped_device_key_nonce)?;
        let device_key =
            keywrap::unwrap_key(&wrapped_device_key, &wrapped_device_key_nonce, &master_key)?;

        Ok(Self {
            dek: SensitiveString::new(dek.to_vec()),
            device_key,
            recovery_key: None,
        })
    }
}

fn derive_master_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    let key_bytes = argon2id::derive_key(password, salt).map_err(|e| KeyringError::Crypto {
        context: e.to_string(),
    })?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    key
}

fn decode_fixed_salt(encoded: &str) -> Result<[u8; 16]> {
    let bytes = STANDARD.decode(encoded).map_err(|e| KeyringError::Crypto {
        context: format!("Invalid salt encoding: {}", e),
    })?;
    if bytes.len() != 16 {
        return Err(KeyringError::Crypto {
            context: "Invalid salt length".to_string(),
        });
    }
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&bytes);
    Ok(salt)
}

fn decode_fixed_nonce(encoded: &str) -> Result<[u8; 12]> {
    let bytes = STANDARD.decode(encoded).map_err(|e| KeyringError::Crypto {
        context: format!("Invalid nonce encoding: {}", e),
    })?;
    if bytes.len() != 12 {
        return Err(KeyringError::Crypto {
            context: "Invalid nonce length".to_string(),
        });
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&bytes);
    Ok(nonce)
}

fn hash_recovery_key(mnemonic: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(mnemonic.as_bytes());
    let digest = hasher.finalize();
    STANDARD.encode(digest.as_slice())
}

/// Verify a recovery key against its hash
///
/// This function computes the hash of the provided mnemonic and compares it
/// with the stored hash to verify the recovery key is correct.
pub fn verify_recovery_key(mnemonic: &str, stored_hash: &str) -> bool {
    let computed_hash = hash_recovery_key(mnemonic);
    computed_hash == stored_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_recovery_key_valid() {
        // Generate a valid mnemonic
        let mnemonic = bip39::generate_mnemonic(24).unwrap();
        let stored_hash = hash_recovery_key(&mnemonic);

        assert!(verify_recovery_key(&mnemonic, &stored_hash));
    }

    #[test]
    fn test_verify_recovery_key_invalid_different_mnemonic() {
        let mnemonic1 = bip39::generate_mnemonic(24).unwrap();
        let mnemonic2 = bip39::generate_mnemonic(24).unwrap();
        let stored_hash = hash_recovery_key(&mnemonic1);

        // Different mnemonic should not match
        assert!(!verify_recovery_key(&mnemonic2, &stored_hash));
    }

    #[test]
    fn test_verify_recovery_key_invalid_empty_mnemonic() {
        let mnemonic = bip39::generate_mnemonic(24).unwrap();
        let stored_hash = hash_recovery_key(&mnemonic);

        // Empty mnemonic should not match
        assert!(!verify_recovery_key("", &stored_hash));
    }

    #[test]
    fn test_verify_recovery_key_invalid_wrong_hash() {
        let mnemonic = bip39::generate_mnemonic(24).unwrap();

        // Random hash that won't match
        let wrong_hash = "dGVzdGhhc2ggd2hvbmNoIG1hdGNoZXMK";

        assert!(!verify_recovery_key(&mnemonic, wrong_hash));
    }

    #[test]
    fn test_hash_recovery_key_consistent() {
        let mnemonic = bip39::generate_mnemonic(12).unwrap();
        let hash1 = hash_recovery_key(&mnemonic);
        let hash2 = hash_recovery_key(&mnemonic);

        // Same mnemonic should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_recovery_key_different_for_different_mnemonics() {
        let mnemonic1 = bip39::generate_mnemonic(24).unwrap();
        let mnemonic2 = bip39::generate_mnemonic(24).unwrap();

        let hash1 = hash_recovery_key(&mnemonic1);
        let hash2 = hash_recovery_key(&mnemonic2);

        // Different mnemonics should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_decode_fixed_salt_valid() {
        let salt = argon2id::generate_salt();
        let encoded = STANDARD.encode(salt);

        let decoded = decode_fixed_salt(&encoded).unwrap();
        assert_eq!(decoded, salt);
    }

    #[test]
    fn test_decode_fixed_salt_invalid_length() {
        // Too short - only 8 bytes instead of 16
        let invalid = "dGVzdHNhbHQ";

        let result = decode_fixed_salt(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_fixed_nonce_valid() {
        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce);
        let encoded = STANDARD.encode(nonce);

        let decoded = decode_fixed_nonce(&encoded).unwrap();
        assert_eq!(decoded, nonce);
    }

    #[test]
    fn test_decode_fixed_nonce_invalid_length() {
        // Too long - 16 bytes instead of 12
        let too_long = STANDARD.encode([1u8; 16]);

        let result = decode_fixed_nonce(&too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random_key_different_each_time() {
        let key1 = generate_random_key();
        let key2 = generate_random_key();

        // Keys should be different (random)
        assert_ne!(key1.to_vec(), key2.to_vec());
    }

    #[test]
    fn test_generate_random_key_length() {
        let key = generate_random_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_master_key_consistent() {
        let password = "test-password-123";
        let salt = argon2id::generate_salt();

        let key1 = derive_master_key(password, &salt).unwrap();
        let key2 = derive_master_key(password, &salt).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_master_key_different_for_different_passwords() {
        let salt = argon2id::generate_salt();

        let key1 = derive_master_key("password1", &salt).unwrap();
        let key2 = derive_master_key("password2", &salt).unwrap();

        assert_ne!(key1.to_vec(), key2.to_vec());
    }

    #[test]
    fn test_keystore_get_dek() {
        let keystore = KeyStore {
            dek: SensitiveString::new(vec![1u8, 2, 3, 4, 5, 6, 7, 8]),
            device_key: [0u8; 32],
            recovery_key: None,
        };

        let dek_slice = keystore.get_dek();
        assert_eq!(dek_slice, &[1u8, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_keystore_initialization_creates_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("keystore.json");

        let keystore = KeyStore::initialize(&keystore_path, "test-password").unwrap();

        // Verify keystore data
        assert_eq!(keystore.get_dek().len(), 32);
        assert_eq!(keystore.device_key.len(), 32);
        assert!(keystore.recovery_key.is_some());

        // Verify file was created
        assert!(keystore_path.exists());

        // Verify file can be unlocked
        let unlocked = KeyStore::unlock(&keystore_path, "test-password").unwrap();
        assert_eq!(unlocked.get_dek().len(), 32);
    }

    #[test]
    fn test_keystore_unlock_with_correct_password() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("keystore.json");

        // Initialize keystore
        KeyStore::initialize(&keystore_path, "my-secure-password").unwrap();

        // Unlock with correct password
        let unlocked = KeyStore::unlock(&keystore_path, "my-secure-password").unwrap();
        assert_eq!(unlocked.get_dek().len(), 32);
        assert_eq!(unlocked.device_key.len(), 32);
    }

    #[test]
    fn test_keystore_unlock_with_wrong_password_fails() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("keystore.json");

        // Initialize keystore
        KeyStore::initialize(&keystore_path, "correct-password").unwrap();

        // Try to unlock with wrong password
        let result = KeyStore::unlock(&keystore_path, "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_keystore_unlock_nonexistent_file_fails() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("nonexistent.json");

        let result = KeyStore::unlock(&keystore_path, "any-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_keystore_recovery_key_is_valid_bip39() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("keystore.json");

        let keystore = KeyStore::initialize(&keystore_path, "password").unwrap();
        let recovery_key = keystore.recovery_key.as_ref().unwrap();

        // Verify recovery key is a valid 24-word BIP39 mnemonic
        let words: Vec<&str> = recovery_key.split_whitespace().collect();
        assert_eq!(words.len(), 24);

        // Verify it can be validated
        assert!(bip39::validate_mnemonic(recovery_key).unwrap());
    }

    #[test]
    fn test_keystore_unlock_different_password_than_initialization() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let keystore_path = temp_dir.path().join("keystore.json");

        // Initialize with one password
        KeyStore::initialize(&keystore_path, "password1").unwrap();

        // Try to unlock with different password
        let result = KeyStore::unlock(&keystore_path, "password2");
        assert!(result.is_err());
    }
}
