//! Cryptographic primitives for key derivation and encryption

pub mod argon2id;
pub mod aes256gcm;
pub mod keywrap;
pub mod bip39;
pub mod keystore;
pub mod record;

use crate::error::KeyringError;
use anyhow::Result;
use zeroize::Zeroize;

/// High-level crypto manager for key operations
pub struct CryptoManager {
    master_key: Option<Vec<u8>>,
    salt: Option<[u8; 16]>,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            master_key: None,
            salt: None,
        }
    }

    /// Initialize with a master password
    pub fn initialize(&mut self, password: &str) -> Result<(), KeyringError> {
        let salt = argon2id::generate_salt();
        self.master_key = Some(argon2id::derive_key(password, &salt)?);
        self.salt = Some(salt);
        Ok(())
    }

    /// Initialize with existing salt (for loading from storage)
    pub fn initialize_with_salt(&mut self, password: &str, salt: [u8; 16]) -> Result<(), KeyringError> {
        self.master_key = Some(argon2id::derive_key(password, &salt)?);
        self.salt = Some(salt);
        Ok(())
    }

    /// Get the salt for persistence
    pub fn get_salt(&self) -> Option<[u8; 16]> {
        self.salt
    }

    /// Encrypt data using the current master key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), KeyringError> {
        let key = self.master_key.as_ref()
            .ok_or_else(|| KeyringError::Crypto { context: "Not initialized".to_string() })?;
        let key_array: [u8; 32] = key.as_slice().try_into()
            .map_err(|_| KeyringError::Crypto { context: "Invalid key length".to_string() })?;
        aes256gcm::encrypt(plaintext, &key_array)
            .map_err(|e| KeyringError::Crypto { context: e.to_string() })
    }

    /// Decrypt data using the current master key
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, KeyringError> {
        let key = self.master_key.as_ref()
            .ok_or_else(|| KeyringError::Crypto { context: "Not initialized".to_string() })?;
        let key_array: [u8; 32] = key.as_slice().try_into()
            .map_err(|_| KeyringError::Crypto { context: "Invalid key length".to_string() })?;
        aes256gcm::decrypt(ciphertext, nonce, &key_array)
            .map_err(|e| KeyringError::Crypto { context: e.to_string() })
    }

    /// Derive a sub-key using HKDF-like approach
    pub fn derive_sub_key(&self, context: &[u8]) -> Result<[u8; 32], KeyringError> {
        let master = self.master_key.as_ref()
            .ok_or_else(|| KeyringError::Crypto { context: "Not initialized".to_string() })?;

        // Simple sub-key derivation: hash(master || context)
        use sha2::Sha256;
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(master);
        hasher.update(context);
        let result = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(result.as_slice());
        Ok(key)
    }

    /// Securely clear sensitive data
    pub fn clear(&mut self) {
        if let Some(mut key) = self.master_key.take() {
            key.zeroize();
        }
        self.salt = None;
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.master_key.is_some()
    }

    /// Generate a random password with specified length
    pub fn generate_random_password(&self, length: usize) -> Result<String, KeyringError> {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";

        if length < 4 {
            return Err(KeyringError::InvalidInput {
                context: "Password length must be at least 4 characters".to_string(),
            });
        }
        if length > 128 {
            return Err(KeyringError::InvalidInput {
                context: "Password length cannot exceed 128 characters".to_string(),
            });
        }

        let mut rng = rand::thread_rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Ok(password)
    }

    /// Generate a memorable password using word-based approach
    pub fn generate_memorable_password(&self, word_count: usize) -> Result<String, KeyringError> {
        const WORDS: &[&str] = &[
            "correct", "horse", "battery", "staple", "apple", "banana", "cherry", "dragon",
            "elephant", "flower", "garden", "house", "island", "jungle", "kangaroo", "lemon",
            "mountain", "nectar", "orange", "piano", "queen", "river", "sunshine", "tiger",
            "umbrella", "violet", "whale", "xylophone", "yellow", "zebra", "castle", "desert",
            "eagle", "forest", "giraffe", "harbor", "igloo", "journey", "kingdom", "lantern",
            "meadow", "night", "ocean", "planet", "quartz", "rainbow", "star", "tower",
            "universe", "valley", "wave", "crystal", "year", "zen", "bridge", "cloud",
            "diamond", "emerald", "fountain", "galaxy", "horizon", "infinity", "jewel",
        ];

        if word_count < 3 {
            return Err(KeyringError::InvalidInput {
                context: "Word count must be at least 3".to_string(),
            });
        }
        if word_count > 12 {
            return Err(KeyringError::InvalidInput {
                context: "Word count cannot exceed 12".to_string(),
            });
        }

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let selected: Vec<&str> = WORDS.choose_multiple(&mut rng, word_count).copied().collect();

        Ok(selected.join("-"))
    }

    /// Generate a numeric PIN
    pub fn generate_pin(&self, length: usize) -> Result<String, KeyringError> {
        use rand::Rng;

        if length < 4 {
            return Err(KeyringError::InvalidInput {
                context: "PIN length must be at least 4 digits".to_string(),
            });
        }
        if length > 16 {
            return Err(KeyringError::InvalidInput {
                context: "PIN length cannot exceed 16 digits".to_string(),
            });
        }

        let mut rng = rand::thread_rng();
        let pin: String = (0..length)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        Ok(pin)
    }
}

impl Drop for CryptoManager {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_manager_salt_persistence() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("test-password").unwrap();

        // Get the salt for storage
        let salt = crypto.get_salt().unwrap();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn test_crypto_manager_reinitialization() {
        let mut crypto = CryptoManager::new();

        crypto.initialize("password1").unwrap();
        let ciphertext = crypto.encrypt(b"data").unwrap();

        crypto.initialize("password2").unwrap();
        // Old ciphertext should not decrypt with new key
        let result = crypto.decrypt(&ciphertext.0, &ciphertext.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_crypto_manager_clear() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("password").unwrap();

        assert!(crypto.is_initialized());
        crypto.clear();
        assert!(!crypto.is_initialized());
    }

    #[test]
    fn test_sub_key_derivation() {
        let mut crypto = CryptoManager::new();
        crypto.initialize("password").unwrap();

        let key1 = crypto.derive_sub_key(b"context1").unwrap();
        let key2 = crypto.derive_sub_key(b"context2").unwrap();

        // Different contexts should produce different keys
        assert_ne!(key1.to_vec(), key2.to_vec());
    }
}

// Re-exports for convenience
pub use argon2id::{
    Argon2Params, DeviceCapability,
    derive_key, derive_key_with_params, generate_salt,
    hash_password, verify_password,
    detect_device_capability, verify_params_security,
    PasswordHash,
};
pub use aes256gcm::{encrypt, decrypt, encrypt_with_aad, decrypt_with_aad, EncryptedData};
pub use keywrap::{wrap_key, unwrap_key};
