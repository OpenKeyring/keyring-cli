//! Integration tests for HKDF device key derivation

use keyring_cli::crypto::hkdf::derive_device_key;

#[test]
fn deterministic_derivation_same_inputs_same_output() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    let device_id = "macos-MacBookPro-a1b2c3d4";

    let key1 = derive_device_key(&master_key, device_id);
    let key2 = derive_device_key(&master_key, device_id);

    assert_eq!(key1, key2, "Same inputs should produce same output");
}

#[test]
fn device_id_uniqueness_different_ids_different_keys() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    let key1 = derive_device_key(&master_key, "macos-MacBookPro-a1b2c3d4");
    let key2 = derive_device_key(&master_key, "ios-iPhone15-e5f6g7h8");
    let key3 = derive_device_key(&master_key, "linux-desktop-12345678");

    assert_ne!(
        key1, key2,
        "Different device IDs should produce different keys"
    );
    assert_ne!(
        key1, key3,
        "Different device IDs should produce different keys"
    );
    assert_ne!(
        key2, key3,
        "Different device IDs should produce different keys"
    );
}

#[test]
fn cryptographic_independence_derived_key_different_from_master() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    let device_id = "macos-MacBookPro-a1b2c3d4";

    let derived_key = derive_device_key(&master_key, device_id);

    assert_ne!(
        derived_key.to_vec(),
        master_key.to_vec(),
        "Derived key must be different from master key"
    );
}

#[test]
fn valid_output_length_always_32_bytes() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    // Test with various device IDs
    let key1 = derive_device_key(&master_key, "device-1");
    let key2 = derive_device_key(&master_key, "macos-MacBookPro-a1b2c3d4");
    let key3 = derive_device_key(&master_key, "a");

    assert_eq!(key1.len(), 32, "Derived key must be 32 bytes");
    assert_eq!(key2.len(), 32, "Derived key must be 32 bytes");
    assert_eq!(key3.len(), 32, "Derived key must be 32 bytes");
}

#[test]
fn device_id_boundary_empty_device_id() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    // Empty device ID should still produce a valid key
    let key = derive_device_key(&master_key, "");
    assert_eq!(key.len(), 32, "Empty device ID should produce 32-byte key");
}

#[test]
fn device_id_boundary_long_device_id() {
    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    // Very long device ID (1000 characters)
    let long_device_id = "a".repeat(1000);
    let key = derive_device_key(&master_key, &long_device_id);
    assert_eq!(key.len(), 32, "Long device ID should produce 32-byte key");
}

#[test]
fn integration_derived_key_can_encrypt_decrypt() {
    use keyring_cli::crypto::{decrypt, encrypt};

    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    let device_id = "macos-MacBookPro-a1b2c3d4";

    // Derive device key
    let device_key = derive_device_key(&master_key, device_id);

    // Use derived key to encrypt data
    let plaintext = b"sensitive data that needs encryption";
    let (ciphertext, nonce) =
        encrypt(plaintext, &device_key).expect("Derived key should be able to encrypt");

    // Use derived key to decrypt data
    let decrypted =
        decrypt(&ciphertext, &nonce, &device_key).expect("Derived key should be able to decrypt");

    assert_eq!(
        decrypted.as_slice(),
        plaintext,
        "Decrypted data should match original plaintext"
    );
}

#[test]
fn integration_different_device_keys_produce_different_ciphertexts() {
    use keyring_cli::crypto::encrypt;

    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    let device_key_1 = derive_device_key(&master_key, "device-1");
    let device_key_2 = derive_device_key(&master_key, "device-2");

    let plaintext = b"same plaintext";

    let (ciphertext1, _nonce1) =
        encrypt(plaintext, &device_key_1).expect("Should encrypt with device key 1");
    let (ciphertext2, _nonce2) =
        encrypt(plaintext, &device_key_2).expect("Should encrypt with device key 2");

    assert_ne!(
        ciphertext1, ciphertext2,
        "Different device keys should produce different ciphertexts"
    );
}

#[test]
fn master_key_change_produces_different_device_key() {
    let master_key_1 = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];
    let master_key_2 = [
        0x20, 0x1f, 0x1e, 0x1d, 0x1c, 0x1b, 0x1a, 0x19, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12,
        0x11, 0x10, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
        0x02, 0x01,
    ];

    let device_id = "macos-MacBookPro-a1b2c3d4";

    let key1 = derive_device_key(&master_key_1, device_id);
    let key2 = derive_device_key(&master_key_2, device_id);

    assert_ne!(
        key1, key2,
        "Different master keys should produce different device keys"
    );
}

#[test]
fn hkdf_produces_cryptographically_strong_keys() {
    use sha2::{Digest, Sha256};

    let master_key = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    // Derive keys for multiple similar device IDs
    let key1 = derive_device_key(&master_key, "device-001");
    let key2 = derive_device_key(&master_key, "device-002");
    let key3 = derive_device_key(&master_key, "device-003");

    // Verify keys are different (avalanche effect)
    let hash1 = Sha256::digest(key1);
    let hash2 = Sha256::digest(key2);
    let hash3 = Sha256::digest(key3);

    assert_ne!(
        hash1, hash2,
        "Similar device IDs should produce very different keys"
    );
    assert_ne!(
        hash2, hash3,
        "Similar device IDs should produce very different keys"
    );
    assert_ne!(
        hash1, hash3,
        "Similar device IDs should produce very different keys"
    );

    // Count bit differences (should be ~50% for strong KDF)
    let diff1_2 = count_bit_differences(&key1, &key2);
    let diff2_3 = count_bit_differences(&key2, &key3);
    let diff1_3 = count_bit_differences(&key1, &key3);

    // Each key is 32 bytes = 256 bits, so we expect ~128 bits different (40% minimum threshold)
    assert!(
        diff1_2 > 100,
        "Insufficient bit difference between keys 1 and 2: {}",
        diff1_2
    );
    assert!(
        diff2_3 > 100,
        "Insufficient bit difference between keys 2 and 3: {}",
        diff2_3
    );
    assert!(
        diff1_3 > 100,
        "Insufficient bit difference between keys 1 and 3: {}",
        diff1_3
    );
}

fn count_bit_differences(key1: &[u8; 32], key2: &[u8; 32]) -> i32 {
    let mut differences = 0;
    for (b1, b2) in key1.iter().zip(key2.iter()) {
        let xor = b1 ^ b2;
        differences += xor.count_ones();
    }
    differences as i32
}
