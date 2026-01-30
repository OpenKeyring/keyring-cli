// tests/crypto/keyhierarchy_test.rs
use keyring_cli::crypto::keywrap::KeyHierarchy;
use tempfile::TempDir;
use std::path::PathBuf;

#[test]
fn test_keyhierarchy_save_and_unlock() {
    let temp_dir = TempDir::new().unwrap();
    let key_path: PathBuf = temp_dir.path().join("keys");

    // Setup
    let hierarchy = KeyHierarchy::setup("password123").unwrap();
    let original_master = hierarchy.master_key.0;

    // Save wrapped keys
    hierarchy.save(&key_path).unwrap();

    // Unlock with same password
    let loaded = KeyHierarchy::unlock(&key_path, "password123").unwrap();
    assert_eq!(loaded.master_key.0, original_master);
    assert_eq!(loaded.dek.0, hierarchy.dek.0);
    assert_eq!(loaded.recovery_key.0, hierarchy.recovery_key.0);
}

#[test]
fn test_keyhierarchy_unlock_wrong_password() {
    let temp_dir = TempDir::new().unwrap();
    let key_path: PathBuf = temp_dir.path().join("keys");

    let hierarchy = KeyHierarchy::setup("password123").unwrap();
    hierarchy.save(&key_path).unwrap();

    // Wrong password should fail
    let result = KeyHierarchy::unlock(&key_path, "wrongpassword");
    assert!(result.is_err());
}

#[test]
fn test_keyhierarchy_device_key_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let key_path: PathBuf = temp_dir.path().join("keys");

    let hierarchy = KeyHierarchy::setup("password123").unwrap();
    let original_device_key = hierarchy.device_key.0;

    hierarchy.save(&key_path).unwrap();
    let loaded = KeyHierarchy::unlock(&key_path, "password123").unwrap();

    assert_eq!(loaded.device_key.0, original_device_key);
}

#[test]
fn test_keyhierarchy_saved_files_exist() {
    let temp_dir = TempDir::new().unwrap();
    let key_path: PathBuf = temp_dir.path().join("keys");

    let hierarchy = KeyHierarchy::setup("password123").unwrap();
    hierarchy.save(&key_path).unwrap();

    // Check that wrapped key files exist
    assert!(key_path.join("wrapped_dek").exists());
    assert!(key_path.join("wrapped_recovery").exists());
    assert!(key_path.join("wrapped_device").exists());
}
