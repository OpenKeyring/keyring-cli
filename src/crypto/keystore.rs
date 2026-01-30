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
