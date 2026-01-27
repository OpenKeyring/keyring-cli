//! Onboarding and keystore unlock functionality
//!
//! This module provides functions to ensure the vault is initialized
//! and to unlock the keystore for encryption/decryption operations.

use crate::cli::ConfigManager;
use crate::crypto::{keystore::KeyStore, CryptoManager};
use crate::db::Vault;
use crate::error::{KeyringError, Result};
use crate::onboarding::{is_initialized, initialize_keystore};
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
        std::fs::create_dir_all(parent)
            .map_err(|e| KeyringError::IoError(format!("Failed to create data directory: {}", e)))?;
    }

    // Open vault (creates database if it doesn't exist)
    let _vault = Vault::open(&db_path, "")
        .map_err(|e| KeyringError::Database {
            context: format!("Failed to initialize vault: {}", e),
        })?;

    Ok(())
}

/// Unlock the keystore and return a CryptoManager initialized with DEK
///
/// Prompts for master password, unlocks keystore, and initializes CryptoManager with DEK.
///
/// # Returns
/// Initialized CryptoManager ready for encryption/decryption
pub fn unlock_keystore() -> Result<CryptoManager> {
    let config = ConfigManager::new()?;
    let master_password = prompt_for_master_password()?;
    let keystore_path = config.get_keystore_path();
    
    // Unlock or initialize keystore
    let keystore = if is_initialized(&keystore_path) {
        KeyStore::unlock(&keystore_path, &master_password)?
    } else {
        let keystore = initialize_keystore(&keystore_path, &master_password)?;
        if let Some(recovery_key) = &keystore.recovery_key {
            println!("🔑 Recovery Key (save securely): {}", recovery_key);
        }
        keystore
    };
    
    // Initialize CryptoManager with DEK
    let mut crypto = CryptoManager::new();
    crypto.initialize_with_key(keystore.dek);

    Ok(crypto)
}

/// Prompt user for master password
///
/// First checks OK_MASTER_PASSWORD environment variable for automation/testing.
/// Falls back to interactive prompt using rpassword crate.
fn prompt_for_master_password() -> Result<String> {
    use std::io::Write;

    // Check for master password in environment variable (for testing/automation)
    if let Ok(env_password) = std::env::var("OK_MASTER_PASSWORD") {
        if !env_password.is_empty() {
            return Ok(env_password);
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
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_initialized_creates_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // Set environment variable to use temp directory
        std::env::set_var("OK_DATA_DIR", temp_dir.path().to_str().unwrap());
        
        // This should create the database
        let result = ensure_initialized();
        assert!(result.is_ok());
        
        // Cleanup
        std::env::remove_var("OK_DATA_DIR");
    }
}
