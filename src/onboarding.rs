//! Onboarding helpers for initializing local keystore

use crate::crypto::keystore::KeyStore;
use crate::error::Result;
use std::path::Path;

#[cfg(feature = "test-env")]
use crate::cli::config::OpenKeyringConfig;
#[cfg(feature = "test-env")]
use std::fs;

pub fn is_initialized(keystore_path: &Path) -> bool {
    keystore_path.exists()
}

pub fn initialize_keystore(keystore_path: &Path, master_password: &str) -> Result<KeyStore> {
    KeyStore::initialize(keystore_path, master_password)
}

/// Test helper: Initialize a minimal system for integration tests
/// This creates the minimal required files (keystore.json, config.yaml, database)
#[cfg(feature = "test-env")]
pub fn setup_test_system(config_dir: &Path, data_dir: &Path, master_password: &str) -> Result<()> {
    // Create directories
    fs::create_dir_all(config_dir)?;
    fs::create_dir_all(data_dir)?;

    // Initialize keystore
    let keystore_path = config_dir.join("keystore.json");
    initialize_keystore(&keystore_path, master_password)?;

    // Create default config.yaml
    // Set environment variables before generating default config
    std::env::set_var("OK_DATA_DIR", data_dir);
    let config = OpenKeyringConfig::default();
    let config_yaml = serde_yaml::to_string(&config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
    fs::write(config_dir.join("config.yaml"), config_yaml)?;

    // Create empty database (schema will be initialized on first use)
    fs::write(data_dir.join("passwords.db"), "")?;

    Ok(())
}
