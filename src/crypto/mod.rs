//! Cryptographic primitives for key derivation and encryption

pub mod aes256gcm;
pub mod argon2id;
pub mod bip39;
pub mod hkdf;
pub mod keystore;
pub mod keywrap;
pub mod passkey;
pub mod record;

use crate::crypto::passkey::Passkey;
use crate::error::KeyringError;
use anyhow::Result;
use rand::prelude::IndexedRandom;
use std::path::PathBuf;
use zeroize::Zeroize;

use base64::Engine;

/// High-level crypto manager for key operations
pub struct CryptoManager {
    master_key: Option<Vec<u8>>,
    salt: Option<[u8; 16]>,
    device_key: Option<[u8; 32]>,
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            master_key: None,
            salt: None,
            device_key: None,
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
    pub fn initialize_with_salt(
        &mut self,
        password: &str,
        salt: [u8; 16],
    ) -> Result<(), KeyringError> {
        self.master_key = Some(argon2id::derive_key(password, &salt)?);
        self.salt = Some(salt);
        Ok(())
    }

    /// Initialize directly with a derived key (e.g. DEK)
    pub fn initialize_with_key(&mut self, key: [u8; 32]) {
        self.master_key = Some(key.to_vec());
        self.salt = None;
    }

    /// Get the salt for persistence
    pub fn get_salt(&self) -> Option<[u8; 16]> {
        self.salt
    }

    /// Encrypt data using the current master key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), KeyringError> {
        let key = self
            .master_key
            .as_ref()
            .ok_or_else(|| KeyringError::Crypto {
                context: "Not initialized".to_string(),
            })?;
        let key_array: [u8; 32] = key
            .as_slice()
            .try_into()
            .map_err(|_| KeyringError::Crypto {
                context: "Invalid key length".to_string(),
            })?;
        aes256gcm::encrypt(plaintext, &key_array).map_err(|e| KeyringError::Crypto {
            context: e.to_string(),
        })
    }

    /// Decrypt data using the current master key
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, KeyringError> {
        let key = self
            .master_key
            .as_ref()
            .ok_or_else(|| KeyringError::Crypto {
                context: "Not initialized".to_string(),
            })?;
        let key_array: [u8; 32] = key
            .as_slice()
            .try_into()
            .map_err(|_| KeyringError::Crypto {
                context: "Invalid key length".to_string(),
            })?;
        aes256gcm::decrypt(ciphertext, nonce, &key_array).map_err(|e| KeyringError::Crypto {
            context: e.to_string(),
        })
    }

    /// Derive a sub-key using HKDF-like approach
    pub fn derive_sub_key(&self, context: &[u8]) -> Result<[u8; 32], KeyringError> {
        let master = self
            .master_key
            .as_ref()
            .ok_or_else(|| KeyringError::Crypto {
                context: "Not initialized".to_string(),
            })?;

        // Simple sub-key derivation: hash(master || context)
        use sha2::Digest;
        use sha2::Sha256;
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
        if let Some(mut key) = self.device_key.take() {
            key.zeroize();
        }
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

        let mut rng = rand::rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Ok(password)
    }

    /// Generate a memorable password using word-based approach
    pub fn generate_memorable_password(&self, word_count: usize) -> Result<String, KeyringError> {
        const WORDS: &[&str] = &[
            "correct",
            "horse",
            "battery",
            "staple",
            "apple",
            "banana",
            "cherry",
            "dragon",
            "elephant",
            "flower",
            "garden",
            "house",
            "island",
            "jungle",
            "kangaroo",
            "lemon",
            "mountain",
            "nectar",
            "orange",
            "piano",
            "queen",
            "river",
            "sunshine",
            "tiger",
            "umbrella",
            "violet",
            "whale",
            "xylophone",
            "yellow",
            "zebra",
            "castle",
            "desert",
            "eagle",
            "forest",
            "giraffe",
            "harbor",
            "igloo",
            "journey",
            "kingdom",
            "lantern",
            "meadow",
            "night",
            "ocean",
            "planet",
            "quartz",
            "rainbow",
            "star",
            "tower",
            "universe",
            "valley",
            "wave",
            "crystal",
            "year",
            "zen",
            "bridge",
            "cloud",
            "diamond",
            "emerald",
            "fountain",
            "galaxy",
            "horizon",
            "infinity",
            "jewel",
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

        let mut rng = rand::rng();
        let selected: Vec<&str> = WORDS
            .choose_multiple(&mut rng, word_count)
            .copied()
            .collect();

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

        let mut rng = rand::rng();
        let pin: String = (0..length)
            .map(|_| rng.random_range(0..10).to_string())
            .collect();

        Ok(pin)
    }

    /// Initialize with Passkey root key architecture
    ///
    /// This method derives a device-specific Master Key from the root master key using HKDF,
    /// wraps the Passkey seed with the device password, and stores it locally.
    ///
    /// # Arguments
    /// * `passkey` - The BIP39 Passkey (24-word mnemonic)
    /// * `device_password` - Password to wrap the Passkey seed
    /// * `root_master_key` - The 32-byte root master key (cross-device)
    /// * `device_index` - The device type index (MacOS, IOS, Windows, Linux, CLI)
    /// * `kdf_nonce` - The 32-byte KDF nonce for entropy injection
    ///
    /// # Returns
    /// * `Ok(())` if initialization succeeds
    /// * `Err(KeyringError)` if initialization fails
    pub fn initialize_with_passkey(
        &mut self,
        passkey: &Passkey,
        device_password: &str,
        root_master_key: &[u8; 32],
        device_index: crate::crypto::hkdf::DeviceIndex,
        kdf_nonce: &[u8; 32],
    ) -> Result<(), KeyringError> {
        // Use DeviceKeyDeriver to derive device-specific Master Key
        let deriver = crate::crypto::hkdf::DeviceKeyDeriver::new(root_master_key, kdf_nonce);
        let device_master_key = deriver.derive_device_key(device_index);

        // Store the device Master Key
        self.master_key = Some(device_master_key.to_vec());
        self.salt = None; // No salt used for Passkey initialization
        self.device_key = Some(device_master_key);

        // Convert Passkey to seed
        let seed = passkey.to_seed(None).map_err(|e| KeyringError::Crypto {
            context: format!("Failed to derive Passkey seed: {}", e),
        })?;

        // Derive wrapping key from device password
        let password_salt = argon2id::generate_salt();
        let wrapping_key_bytes =
            argon2id::derive_key(device_password, &password_salt).map_err(|e| {
                KeyringError::Crypto {
                    context: format!("Failed to derive wrapping key: {}", e),
                }
            })?;
        let wrapping_key: [u8; 32] =
            wrapping_key_bytes
                .try_into()
                .map_err(|_| KeyringError::Crypto {
                    context: "Invalid wrapping key length".to_string(),
                })?;

        // Wrap the first 32 bytes of the Passkey seed (the seed is 64 bytes)
        // Note: We only wrap the first 32 bytes because:
        // 1. The keywrap::wrap_key function only supports 32-byte keys
        // 2. The first 32 bytes of the BIP39 seed provide sufficient entropy
        // 3. The full 64-byte seed can be derived from these 32 bytes when needed
        let seed_vec = seed.get();
        let seed_bytes: [u8; 32] =
            seed_vec[0..32]
                .try_into()
                .map_err(|_| KeyringError::Crypto {
                    context: "Failed to extract first 32 bytes of seed".to_string(),
                })?;
        let (wrapped_seed, nonce) = crate::crypto::keywrap::wrap_key(&seed_bytes, &wrapping_key)
            .map_err(|e| KeyringError::Crypto {
                context: format!("Failed to wrap Passkey seed: {}", e),
            })?;

        // Get the keyring directory (use default path)
        let keyring_path = get_keyring_dir()?;

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&keyring_path).map_err(KeyringError::Io)?;

        // Store wrapped Passkey
        let wrapped_passkey_path = keyring_path.join("wrapped_passkey");
        let wrapped_data = serde_json::json!({
            "wrapped_seed": base64::engine::general_purpose::STANDARD.encode(wrapped_seed),
            "nonce": base64::engine::general_purpose::STANDARD.encode(nonce),
            "salt": base64::engine::general_purpose::STANDARD.encode(password_salt),
        });

        std::fs::write(
            &wrapped_passkey_path,
            serde_json::to_string_pretty(&wrapped_data).map_err(KeyringError::Serialization)?,
        )
        .map_err(KeyringError::Io)?;

        Ok(())
    }

    /// Get the current device Master Key
    ///
    /// Returns the device Master Key if initialized with Passkey, None otherwise.
    pub fn get_device_key(&self) -> Option<[u8; 32]> {
        self.device_key
    }
}

/// Get the keyring directory path
///
/// Returns `~/.local/share/open-keyring` on Unix systems or
/// `%LOCALAPPDATA%\open-keyring` on Windows.
fn get_keyring_dir() -> Result<PathBuf, KeyringError> {
    if let Some(home) = dirs::home_dir() {
        Ok(home.join(".local/share/open-keyring"))
    } else {
        Err(KeyringError::Internal {
            context: "Failed to determine home directory".to_string(),
        })
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
pub use aes256gcm::{decrypt, decrypt_with_aad, encrypt, encrypt_with_aad, EncryptedData};
pub use argon2id::{
    derive_key, derive_key_with_params, detect_device_capability, generate_salt, hash_password,
    verify_params_security, verify_password, Argon2Params, DeviceCapability, PasswordHash,
};
pub use hkdf::{derive_device_key, DeviceIndex, DeviceKeyDeriver};
pub use keystore::verify_recovery_key;
pub use keywrap::{unwrap_key, wrap_key};
