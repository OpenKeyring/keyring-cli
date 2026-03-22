//! Onboarding and keystore unlock functionality
//!
//! This module provides functions to ensure the vault is initialized
//! and to unlock the keystore for encryption/decryption operations.

use crate::cli::ConfigManager;
use crate::crypto::{hkdf::DeviceIndex, keystore::KeyStore, CryptoManager};
use crate::db::Vault;
use crate::error::{KeyringError, Result};
use crate::onboarding::is_initialized;
use std::path::PathBuf;

/// Ensure the vault is initialized
///
/// Checks if the database exists and is properly initialized.
/// If not, creates the database and initializes it.
///
/// # Returns
/// Ok(()) if initialized, Error if initialization fails
pub fn ensure_initialized() -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            KeyringError::IoError(format!("Failed to create data directory: {}", e))
        })?;
    }

    // Open vault (creates database if it doesn't exist)
    let _vault = Vault::open(&db_path, "").map_err(|e| KeyringError::Database {
        context: format!("Failed to initialize vault: {}", e),
    })?;

    Ok(())
}

/// Check if this is first-time use (keystore doesn't exist)
pub fn is_first_time() -> Result<bool> {
    let config = ConfigManager::new()?;
    let keystore_path = config.get_keystore_path();
    Ok(!is_initialized(&keystore_path))
}

/// Check if wrapped_passkey file exists
fn has_wrapped_passkey() -> bool {
    dirs::data_local_dir()
        .map(|p| p.join("open-keyring/wrapped_passkey"))
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Unlock the keystore and return a CryptoManager initialized with DEK
///
/// First checks for wrapped_passkey (from recovery), then falls back to keystore.
/// Prompts for master password, unlocks keystore, and initializes CryptoManager with DEK.
///
/// # Returns
/// Initialized CryptoManager ready for encryption/decryption
pub fn unlock_keystore() -> Result<CryptoManager> {
    let config = ConfigManager::new()?;
    let keystore_path = config.get_keystore_path();

    // Check if this is first-time use
    if !is_initialized(&keystore_path) && !has_wrapped_passkey() {
        return Err(KeyringError::AuthenticationFailed {
            reason: "Keystore not initialized. Please run 'ok wizard' to set up OpenKeyring for the first time.".to_string(),
        });
    }

    let master_password = prompt_for_master_password()?;

    // Try wrapped_passkey first (from recover command)
    if has_wrapped_passkey() {
        let mut crypto = CryptoManager::new();
        match crypto.initialize_with_wrapped_passkey(&master_password, DeviceIndex::CLI) {
            Ok(()) => return Ok(crypto),
            Err(KeyringError::NotFound { .. }) => {
                // wrapped_passkey not found, fall through to keystore
            }
            Err(e) => {
                // If wrapped_passkey exists but fails to decrypt, it might be wrong password
                // Try keystore as fallback
                if is_initialized(&keystore_path) {
                    // Fall through to keystore unlock
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Fallback: Unlock keystore with password
    if is_initialized(&keystore_path) {
        let keystore = KeyStore::unlock(&keystore_path, &master_password)?;

        // Initialize CryptoManager with DEK
        let mut crypto = CryptoManager::new();
        let dek = keystore.get_dek();
        let dek_array: [u8; 32] = dek.try_into().map_err(|_| KeyringError::Crypto {
            context: "Invalid DEK length: expected 32 bytes".to_string(),
        })?;
        crypto.initialize_with_key(dek_array);

        Ok(crypto)
    } else {
        Err(KeyringError::AuthenticationFailed {
            reason: "Keystore not initialized. Please run 'ok wizard' to set up OpenKeyring."
                .to_string(),
        })
    }
}

/// Prompt user for master password
///
/// First checks OK_MASTER_PASSWORD environment variable for automation/testing
/// (only when test-env feature is enabled).
/// Falls back to interactive prompt using rpassword crate.
fn prompt_for_master_password() -> Result<String> {
    use std::io::Write;

    // Check for master password in environment variable (for testing/automation)
    // ONLY available when test-env feature is enabled
    #[cfg(feature = "test-env")]
    {
        if let Ok(env_password) = std::env::var("OK_MASTER_PASSWORD") {
            if !env_password.is_empty() {
                return Ok(env_password);
            }
        }
    }

    // Interactive prompt
    use rpassword::read_password;
    print!("🔐 Enter master password: ");
    let _ = std::io::stdout().flush();

    let password = read_password()
        .map_err(|e| KeyringError::IoError(format!("Failed to read password: {}", e)))?;

    if password.is_empty() {
        return Err(KeyringError::AuthenticationFailed {
            reason: "Master password cannot be empty".to_string(),
        });
    }

    Ok(password)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "test-env")]
    #[test]
    fn test_ensure_initialized_creates_database() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Set environment variables to use temp directory
        std::env::set_var("OK_DATA_DIR", temp_dir.path().to_str().unwrap());
        std::env::set_var(
            "OK_CONFIG_DIR",
            temp_dir.path().join("config").to_str().unwrap(),
        );

        // This should create the database
        let result = super::ensure_initialized();
        assert!(
            result.is_ok(),
            "ensure_initialized should succeed: {:?}",
            result
        );

        // Cleanup
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_CONFIG_DIR");
    }
}
