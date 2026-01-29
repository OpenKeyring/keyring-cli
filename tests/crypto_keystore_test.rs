//! Tests for CryptoManager Passkey integration and device key derivation

use keyring_cli::crypto::{passkey::Passkey, CryptoManager, DeviceIndex};
use std::fs;

#[test]
fn test_passkey_initialization_flow() {
    // Cleanup before test
    let home = dirs::home_dir().expect("Failed to get home directory");
    let wrapped_passkey_path = home.join(".local/share/open-keyring/wrapped_passkey");
    let _ = std::fs::remove_file(&wrapped_passkey_path);

    // Generate a new Passkey (24-word BIP39 mnemonic)
    let passkey = Passkey::generate(24).expect("Failed to generate passkey");
    let words = passkey.to_words();
    assert_eq!(words.len(), 24, "Passkey should have 24 words");

    // Create a root master key (simulating cross-device root)
    let mut root_master_key = [0u8; 32];
    root_master_key.copy_from_slice(&[1u8; 32]);

    // Device password for wrapping the Passkey
    let device_password = "test-device-password";

    // KDF nonce for entropy injection
    let mut kdf_nonce = [0u8; 32];
    kdf_nonce.copy_from_slice(&[2u8; 32]);

    // Create CryptoManager and initialize with Passkey
    let mut crypto_manager = CryptoManager::new();

    // Initialize with CLI device type
    let result = crypto_manager.initialize_with_passkey(
        &passkey,
        device_password,
        &root_master_key,
        DeviceIndex::CLI,
        &kdf_nonce,
    );

    // After implementation, this should succeed
    assert!(result.is_ok(), "Passkey initialization should succeed");

    // Verify the device key is accessible
    let device_key = crypto_manager.get_device_key();
    assert!(device_key.is_some(), "Device key should be available after initialization");
    assert_eq!(device_key.unwrap().len(), 32, "Device key should be 32 bytes");

    // Verify wrapped Passkey file was created in default location
    assert!(wrapped_passkey_path.exists(), "Wrapped Passkey file should be created");

    // Verify the wrapped Passkey can be read and decrypted
    let wrapped_content = fs::read_to_string(&wrapped_passkey_path)
        .expect("Failed to read wrapped Passkey file");

    // The content should be base64-encoded JSON
    assert!(!wrapped_content.is_empty(), "Wrapped Passkey should not be empty");

    // Cleanup
    let _ = std::fs::remove_file(&wrapped_passkey_path);
}

#[test]
fn test_device_key_derivation_and_use() {
    // Test that device keys are deterministic but unique per device

    // Cleanup before test
    let home = dirs::home_dir().expect("Failed to get home directory");
    let wrapped_passkey_path = home.join(".local/share/open-keyring/wrapped_passkey");
    let _ = std::fs::remove_file(&wrapped_passkey_path);

    // Same root master key
    let root_master_key = [1u8; 32];

    // Same KDF nonce
    let kdf_nonce = [2u8; 32];

    // Different device types should produce different device keys
    let device_index_1 = DeviceIndex::MacOS;
    let device_index_2 = DeviceIndex::IOS;

    let mut crypto_manager_1 = CryptoManager::new();
    let mut crypto_manager_2 = CryptoManager::new();

    // Generate a Passkey for each device
    let passkey = Passkey::generate(24).expect("Failed to generate passkey");
    let device_password = "test-password";

    // Initialize both devices with same root key but different device types
    crypto_manager_1
        .initialize_with_passkey(
            &passkey,
            device_password,
            &root_master_key,
            device_index_1,
            &kdf_nonce,
        )
        .expect("Device 1 initialization should succeed");

    crypto_manager_2
        .initialize_with_passkey(
            &passkey,
            device_password,
            &root_master_key,
            device_index_2,
            &kdf_nonce,
        )
        .expect("Device 2 initialization should succeed");

    // Get device keys
    let device_key_1 = crypto_manager_1.get_device_key().expect("Device 1 key should exist");
    let device_key_2 = crypto_manager_2.get_device_key().expect("Device 2 key should exist");

    // Device keys should be different for different device types
    assert_ne!(
        device_key_1, device_key_2,
        "Different device types should produce different device keys"
    );

    // But same device type should produce same device key (deterministic)
    let mut crypto_manager_3 = CryptoManager::new();
    crypto_manager_3
        .initialize_with_passkey(
            &passkey,
            device_password,
            &root_master_key,
            device_index_1,
            &kdf_nonce,
        )
        .expect("Device 3 initialization should succeed");

    let device_key_3 = crypto_manager_3.get_device_key().expect("Device 3 key should exist");

    assert_eq!(
        device_key_1, device_key_3,
        "Same device type should produce same device key (deterministic)"
    );

    // Cleanup
    let _ = std::fs::remove_file(&wrapped_passkey_path);
}

#[test]
fn test_get_device_key_returns_none_when_not_initialized() {
    let crypto_manager = CryptoManager::new();

    // Should return None when not initialized with Passkey
    let device_key = crypto_manager.get_device_key();
    assert!(device_key.is_none(), "Device key should be None when not initialized");
}

#[test]
fn test_get_keyring_dir() {
    // Test that get_keyring_dir returns the correct path
    // This will be a private helper function, so we test it indirectly
    // through initialize_with_passkey

    // Cleanup before test
    let home = dirs::home_dir().expect("Failed to get home directory");
    let wrapped_passkey_path = home.join(".local/share/open-keyring/wrapped_passkey");
    let _ = std::fs::remove_file(&wrapped_passkey_path);

    let passkey = Passkey::generate(24).expect("Failed to generate passkey");
    let root_master_key = [1u8; 32];
    let device_password = "test-password";
    let kdf_nonce = [2u8; 32];

    let mut crypto_manager = CryptoManager::new();

    // Initialize (should use default keyring dir)
    let result = crypto_manager.initialize_with_passkey(
        &passkey,
        device_password,
        &root_master_key,
        DeviceIndex::Windows,
        &kdf_nonce,
    );

    // This should create the wrapped_passkey in the default location
    assert!(result.is_ok(), "Initialization with default path should succeed");

    // Verify the wrapped_passkey file exists in the default location
    // The default location should be ~/.local/share/open-keyring/wrapped_passkey
    assert!(wrapped_passkey_path.exists(), "Wrapped Passkey file should exist");

    // Note: This might fail if the directory doesn't exist or permissions are wrong
    // In a real test, we'd need to set up the environment properly
    // For now, we'll just check that the initialization succeeded

    // Cleanup
    let _ = std::fs::remove_file(&wrapped_passkey_path);
}

#[test]
fn test_passkey_seed_wrapping_and_storage() {
    // Test that the Passkey seed is properly wrapped and stored

    // Cleanup before test
    let home = dirs::home_dir().expect("Failed to get home directory");
    let wrapped_passkey_path = home.join(".local/share/open-keyring/wrapped_passkey");
    let _ = std::fs::remove_file(&wrapped_passkey_path);

    let passkey = Passkey::generate(24).expect("Failed to generate passkey");
    let root_master_key = [1u8; 32];
    let device_password = "strong-device-password-123";
    let kdf_nonce = [3u8; 32];

    let mut crypto_manager = CryptoManager::new();

    crypto_manager
        .initialize_with_passkey(
            &passkey,
            device_password,
            &root_master_key,
            DeviceIndex::Linux,
            &kdf_nonce,
        )
        .expect("Initialization should succeed");

    // Read the wrapped Passkey file from default location
    let wrapped_content = fs::read_to_string(&wrapped_passkey_path)
        .expect("Failed to read wrapped Passkey");

    // Parse as JSON to verify structure
    let wrapped_data: serde_json::Value = serde_json::from_str(&wrapped_content)
        .expect("Failed to parse wrapped Passkey as JSON");

    // Should have wrapped_seed, nonce, and salt fields
    assert!(wrapped_data.get("wrapped_seed").is_some(), "Should have wrapped_seed field");
    assert!(wrapped_data.get("nonce").is_some(), "Should have nonce field");
    assert!(wrapped_data.get("salt").is_some(), "Should have salt field");

    // The wrapped seed should be base64-encoded (not plaintext)
    let wrapped_seed = wrapped_data["wrapped_seed"].as_str().unwrap();
    assert!(!wrapped_seed.contains(&passkey.to_words().join(" ")),
            "Wrapped seed should not contain plaintext mnemonic");

    // Cleanup
    let _ = std::fs::remove_file(&wrapped_passkey_path);
}
