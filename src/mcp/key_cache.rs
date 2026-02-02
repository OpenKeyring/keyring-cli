//! MCP Key Cache
//!
//! This module provides the key cache for MCP server operations.
//! It wraps the KeyStore::unlock() functionality and provides:
//! - Access to the DEK for decrypting credentials
//! - Signing keys derived from DEK via HKDF for confirmation tokens
//! - Automatic zeroization on drop

use crate::cli::config::ConfigManager;
use crate::crypto::hkdf;
use crate::crypto::keystore::KeyStore;
use std::path::PathBuf;
use zeroize::Zeroize;

use anyhow::Result;

/// MCP key cache - holds decrypted keys in memory
///
/// This cache wraps the KeyStore and provides:
/// - DEK access for credential decryption
/// - Signing keys for confirmation tokens (HKDF derived)
/// - Audit signing key (HKDF derived)
///
/// # Security
///
/// All keys are automatically zeroized on drop using the zeroize crate.
pub struct McpKeyCache {
    /// Decrypted Data Encryption Key from KeyStore
    dek: Option<Vec<u8>>,

    /// Signing key for confirmation tokens (HKDF from DEK, info: "mcp-signing-key")
    signing_key: Option<[u8; 32]>,

    /// Signing key for audit logs (HKDF from DEK, info: "audit-signing-key")
    audit_signing_key: Option<[u8; 32]>,

    /// Path to keystore file (for keeping reference)
    keystore_path: PathBuf,
}

impl McpKeyCache {
    /// Create key cache by unlocking with master password
    ///
    /// This method:
    /// 1. Gets the keystore path from ConfigManager
    /// 2. Unlocks the KeyStore with the master password
    /// 3. Extracts the DEK from the KeyStore
    /// 4. Derives signing keys from DEK using HKDF
    ///
    /// # Arguments
    ///
    /// * `master_password` - The master password used to encrypt the keystore
    ///
    /// # Returns
    ///
    /// Ok(McpKeyCache) if unlock succeeds, Err otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - ConfigManager cannot load configuration
    /// - Keystore file doesn't exist or is corrupted
    /// - Master password is incorrect
    /// - Key derivation fails
    pub fn from_master_password(master_password: &str) -> Result<Self, KeyCacheError> {
        // 1. Get keystore path from config
        let config_manager = ConfigManager::new()
            .map_err(|e| KeyCacheError::Custom(format!("Failed to load config: {}", e)))?;
        let keystore_path = config_manager.get_keystore_path();

        // 2. Unlock the keystore
        let keystore = KeyStore::unlock(&keystore_path, master_password)
            .map_err(|_| KeyCacheError::UnlockFailed)?;

        // 3. Extract DEK from keystore
        let dek = keystore.get_dek().to_vec();

        // 4. Derive signing keys from DEK using HKDF
        let dek_array: [u8; 32] = dek
            .as_slice()
            .try_into()
            .map_err(|_| KeyCacheError::InvalidKeyLength)?;

        let signing_key = hkdf::derive_device_key(&dek_array, "mcp-signing-key");
        let audit_signing_key = hkdf::derive_device_key(&dek_array, "audit-signing-key");

        Ok(Self {
            dek: Some(dek),
            signing_key: Some(signing_key),
            audit_signing_key: Some(audit_signing_key),
            keystore_path,
        })
    }

    /// Get the signing key for confirmation tokens
    ///
    /// This key is used to sign confirmation tokens to prevent tampering.
    pub fn signing_key(&self) -> Result<&[u8; 32], KeyCacheError> {
        self.signing_key
            .as_ref()
            .ok_or(KeyCacheError::NotInitialized)
    }

    /// Get the signing key for audit logs
    ///
    /// This key is used to sign audit log entries for integrity verification.
    pub fn audit_signing_key(&self) -> Result<&[u8; 32], KeyCacheError> {
        self.audit_signing_key
            .as_ref()
            .ok_or(KeyCacheError::NotInitialized)
    }

    /// Get the DEK for credential decryption
    ///
    /// # Returns
    ///
    /// A reference to the DEK byte slice
    pub fn dek(&self) -> Result<&[u8], KeyCacheError> {
        self.dek
            .as_deref()
            .ok_or(KeyCacheError::NotInitialized)
    }

    /// Get the keystore path (for reference/logging)
    pub fn keystore_path(&self) -> &PathBuf {
        &self.keystore_path
    }
}

impl Drop for McpKeyCache {
    fn drop(&mut self) {
        // Zeroize sensitive fields on drop
        if let Some(mut dek) = self.dek.take() {
            dek.zeroize();
        }
        if let Some(mut signing_key) = self.signing_key.take() {
            signing_key.zeroize();
        }
        if let Some(mut audit_signing_key) = self.audit_signing_key.take() {
            audit_signing_key.zeroize();
        }
    }
}

/// Errors that can occur when working with the key cache
#[derive(Debug, thiserror::Error)]
pub enum KeyCacheError {
    #[error("Failed to unlock keystore - wrong password?")]
    UnlockFailed,

    #[error("Key cache not initialized")]
    NotInitialized,

    #[error("Invalid key length - expected 32 bytes")]
    InvalidKeyLength,

    #[error("Key cache error: {0}")]
    Custom(String),
}

impl From<anyhow::Error> for KeyCacheError {
    fn from(err: anyhow::Error) -> Self {
        KeyCacheError::Custom(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests require an initialized keystore file
    // These are basic unit tests for the structure

    #[test]
    fn test_key_cache_error_display() {
        let err = KeyCacheError::UnlockFailed;
        assert!(err.to_string().contains("wrong password"));
    }

    #[test]
    fn test_key_cache_not_initialized() {
        let cache = McpKeyCache {
            dek: None,
            signing_key: None,
            audit_signing_key: None,
            keystore_path: PathBuf::from("/test/keystore.json"),
        };

        assert!(matches!(cache.dek(), Err(KeyCacheError::NotInitialized)));
        assert!(matches!(
            cache.signing_key(),
            Err(KeyCacheError::NotInitialized)
        ));
    }
}
