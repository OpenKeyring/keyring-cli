//! Onboarding helpers for initializing local keystore

use crate::crypto::keystore::KeyStore;
use crate::error::Result;
use std::path::Path;

pub fn is_initialized(keystore_path: &Path) -> bool {
    keystore_path.exists()
}

pub fn initialize_keystore(keystore_path: &Path, master_password: &str) -> Result<KeyStore> {
    KeyStore::initialize(keystore_path, master_password)
}
