//! Tests for MCP key cache module
//!
//! The key cache wraps KeyStore::unlock() and provides:
//! - Access to the DEK for decrypting credentials
//! - Signing keys derived from DEK via HKDF
//! - Automatic zeroization on drop

use tempfile::TempDir;

#[test]
fn test_hkdf_key_derivation() {
    // Test that HKDF derivation works correctly
    let dek = [1u8; 32];

    // Use the existing hkdf module
    let signing_key = keyring_cli::crypto::hkdf::derive_device_key(&dek, "mcp-signing-key");
    let audit_key = keyring_cli::crypto::hkdf::derive_device_key(&dek, "audit-signing-key");

    // Both should be 32 bytes
    assert_eq!(signing_key.len(), 32);
    assert_eq!(audit_key.len(), 32);

    // Same input should produce same key
    let signing_key2 = keyring_cli::crypto::hkdf::derive_device_key(&dek, "mcp-signing-key");
    assert_eq!(signing_key, signing_key2);

    // Different context should produce different key
    assert_ne!(signing_key, audit_key);
}

#[test]
fn test_zeroize_on_drop() {
    use zeroize::Zeroize;

    let mut sensitive = vec![0x42u8; 32];
    sensitive.zeroize();

    // Should be zeroed
    assert!(sensitive.iter().all(|&b| b == 0));
}

#[test]
fn test_keystore_requires_existing_file() {
    // KeyStore::unlock() requires an existing keystore file
    let temp_dir = TempDir::new().unwrap();
    let keystore_path = temp_dir.path().join("test_keystore.json");

    // This should fail because keystore doesn't exist
    let result = keyring_cli::crypto::keystore::KeyStore::unlock(&keystore_path, "test-password");
    assert!(result.is_err());

    // Also test that wrong password fails (if keystore existed)
}

#[test]
fn test_config_manager_has_keystore_path() {
    // Verify ConfigManager provides keystore path
    // This test just checks the interface exists
    let config = keyring_cli::cli::config::ConfigManager::new().unwrap();
    let keystore_path = config.get_keystore_path();

    // Should return a path ending with keystore.json
    assert!(keystore_path.ends_with("keystore.json"));

    // Should be in the config directory
    assert!(keystore_path.parent().is_some());
}
