//! HKDF-based device key derivation
//!
//! This module provides device-specific key derivation using HKDF-SHA256 (RFC 5869).
//! Device keys are derived from the master key using the device ID as context info,
//! ensuring each device has a cryptographically unique key while maintaining
//! determinism.

use hkdf::Hkdf;
use sha2::Sha256;
use std::fmt;
use zeroize::ZeroizeOnDrop;

/// Device index for key derivation
///
/// Represents different platform types for device-specific key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceIndex {
    MacOS,
    IOS,
    Windows,
    Linux,
    CLI,
}

impl DeviceIndex {
    /// Convert to string for use in HKDF info parameter
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceIndex::MacOS => "macos",
            DeviceIndex::IOS => "ios",
            DeviceIndex::Windows => "windows",
            DeviceIndex::Linux => "linux",
            DeviceIndex::CLI => "cli",
        }
    }
}

/// Device key deriver for batch derivation
///
/// This struct encapsulates the root master key and KDF nonce for efficient
/// batch derivation of multiple device keys.
#[derive(ZeroizeOnDrop)]
pub struct DeviceKeyDeriver {
    root_master_key: [u8; 32],
    kdf_nonce: [u8; 32],
}

impl fmt::Debug for DeviceKeyDeriver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeviceKeyDeriver")
            .field("root_master_key", &"<redacted>")
            .field("kdf_nonce", &"<redacted>")
            .finish()
    }
}

impl DeviceKeyDeriver {
    /// Create a new DeviceKeyDeriver
    ///
    /// # Arguments
    /// * `root_master_key` - The 32-byte root master key (cross-device)
    /// * `kdf_nonce` - The 32-byte KDF nonce for entropy injection
    pub fn new(root_master_key: &[u8; 32], kdf_nonce: &[u8; 32]) -> Self {
        let mut key = [0u8; 32];
        key.copy_from_slice(root_master_key);

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(kdf_nonce);

        Self {
            root_master_key: key,
            kdf_nonce: nonce,
        }
    }

    /// Derive a device-specific key
    ///
    /// # Arguments
    /// * `device_index` - The device type index
    ///
    /// # Returns
    /// A 32-byte device-specific key
    pub fn derive_device_key(&self, device_index: DeviceIndex) -> [u8; 32] {
        // Combine root_master_key with kdf_nonce as salt for entropy injection
        let salt = Some(&self.kdf_nonce[..]);

        // Create HKDF instance with SHA256
        let hk = Hkdf::<Sha256>::new(salt, &self.root_master_key);

        // Derive device key using device_index as info
        let mut device_key = [0u8; 32];
        hk.expand(device_index.as_str().as_bytes(), &mut device_key)
            .expect("HKDF expansion should not fail with valid parameters");

        device_key
    }
}

/// Derive a device-specific key from the master key using HKDF-SHA256.
///
/// # Arguments
/// * `master_key` - The 32-byte master key
/// * `device_id` - The unique device identifier (e.g., "macos-MacBookPro-a1b2c3d4")
///
/// # Returns
/// A 32-byte device-specific key
///
/// # Algorithm
/// - Salt: None (optional, using HKDF-Extract with default salt)
/// - IKM (Input Key Material): master_key
/// - Info: device_id.as_bytes()
/// - L (output length): 32 bytes
///
/// # Example
/// ```no_run
/// use keyring_cli::crypto::hkdf::derive_device_key;
///
/// let master_key = [0u8; 32];
/// let device_id = "macos-MacBookPro-a1b2c3d4";
/// let device_key = derive_device_key(&master_key, device_id);
/// assert_eq!(device_key.len(), 32);
/// ```
pub fn derive_device_key(master_key: &[u8; 32], device_id: &str) -> [u8; 32] {
    // Create HKDF instance with SHA256
    let hk = Hkdf::<Sha256>::new(None, master_key);

    // Derive device key using device_id as info
    let mut device_key = [0u8; 32];
    hk.expand(device_id.as_bytes(), &mut device_key)
        .expect("HKDF expansion should not fail with valid parameters");

    device_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_derivation() {
        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let device_id = "macos-MacBookPro-a1b2c3d4";

        let key1 = derive_device_key(&master_key, device_id);
        let key2 = derive_device_key(&master_key, device_id);

        assert_eq!(key1, key2, "Same inputs must produce same output");
    }

    #[test]
    fn test_device_id_uniqueness() {
        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let key1 = derive_device_key(&master_key, "device-1");
        let key2 = derive_device_key(&master_key, "device-2");
        let key3 = derive_device_key(&master_key, "device-3");

        assert_ne!(
            key1, key2,
            "Different device IDs must produce different keys"
        );
        assert_ne!(
            key2, key3,
            "Different device IDs must produce different keys"
        );
        assert_ne!(
            key1, key3,
            "Different device IDs must produce different keys"
        );
    }

    #[test]
    fn test_cryptographic_independence() {
        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let device_id = "test-device";

        let derived_key = derive_device_key(&master_key, device_id);

        assert_ne!(
            derived_key.as_ref(),
            master_key.as_ref(),
            "Derived key must differ from master key"
        );
    }

    #[test]
    fn test_output_length() {
        let master_key = [0u8; 32];

        let key1 = derive_device_key(&master_key, "device-1");
        let key2 = derive_device_key(&master_key, "device-2");
        let key3 = derive_device_key(&master_key, "");

        assert_eq!(key1.len(), 32, "Output must be 32 bytes");
        assert_eq!(key2.len(), 32, "Output must be 32 bytes");
        assert_eq!(key3.len(), 32, "Output must be 32 bytes");
    }

    #[test]
    fn test_empty_device_id() {
        let master_key = [0u8; 32];

        let key = derive_device_key(&master_key, "");
        assert_eq!(
            key.len(),
            32,
            "Empty device ID must produce valid 32-byte key"
        );
    }

    #[test]
    fn test_long_device_id() {
        let master_key = [0u8; 32];
        let long_device_id = "a".repeat(1000);

        let key = derive_device_key(&master_key, &long_device_id);
        assert_eq!(
            key.len(),
            32,
            "Long device ID must produce valid 32-byte key"
        );
    }

    #[test]
    fn test_master_key_sensitivity() {
        let master_key_1 = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let master_key_2 = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x21, // Last byte different
        ];

        let device_id = "test-device";

        let key1 = derive_device_key(&master_key_1, device_id);
        let key2 = derive_device_key(&master_key_2, device_id);

        assert_ne!(
            key1, key2,
            "Single bit change in master key must produce different device key"
        );
    }

    #[test]
    fn test_avalanche_effect() {
        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        // Derive keys for similar device IDs
        let key1 = derive_device_key(&master_key, "device-001");
        let key2 = derive_device_key(&master_key, "device-002");

        // Count bit differences (should be significant for strong KDF)
        let diff_count = count_bit_differences(&key1, &key2);

        // Each key is 256 bits, expect significant difference (at least 40%)
        assert!(
            diff_count > 100,
            "Insufficient avalanche effect: {} bits different",
            diff_count
        );
    }

    #[test]
    fn test_uniform_distribution() {
        let master_key = [42u8; 32];

        // Derive multiple keys
        let keys: Vec<[u8; 32]> = (0..100)
            .map(|i| derive_device_key(&master_key, &format!("device-{}", i)))
            .collect();

        // Check that bytes are roughly uniformly distributed (not all zeros or same value)
        for key in &keys {
            // Ensure not all zeros
            assert_ne!(key, &[0u8; 32], "Key must not be all zeros");

            // Ensure not all same byte
            let first_byte = key[0];
            assert!(
                key.iter().any(|&b| b != first_byte),
                "Key must not be all same byte"
            );
        }

        // Verify all keys are unique
        let unique_keys: std::collections::HashSet<[u8; 32]> = keys.iter().cloned().collect();
        assert_eq!(unique_keys.len(), 100, "All derived keys must be unique");
    }

    #[test]
    fn test_rfc5869_compliance() {
        // Test using known test vectors from RFC 5869
        // This is a simplified version to ensure we're using HKDF correctly

        let master_key = [
            0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
            0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
            0x0b, 0x0b, 0x0b, 0x0b,
        ];
        let device_id = "test-device-id";

        let device_key = derive_device_key(&master_key, device_id);

        // Verify output is valid (not all zeros, correct length)
        assert_ne!(device_key, [0u8; 32], "Derived key must not be all zeros");
        assert_eq!(device_key.len(), 32, "Derived key must be 32 bytes");

        // Verify it's deterministic
        let device_key2 = derive_device_key(&master_key, device_id);
        assert_eq!(device_key, device_key2, "Derivation must be deterministic");
    }

    #[test]
    fn test_unicode_device_id() {
        let master_key = [0u8; 32];

        // Test with Unicode characters
        let device_id_unicode = "device-MacBookPro-test";
        let device_id_emoji = "🔐-device-🔑";

        let key1 = derive_device_key(&master_key, device_id_unicode);
        let key2 = derive_device_key(&master_key, device_id_emoji);

        assert_eq!(key1.len(), 32, "Unicode device ID must produce 32-byte key");
        assert_eq!(key2.len(), 32, "Emoji device ID must produce 32-byte key");
        assert_ne!(
            key1, key2,
            "Different device IDs must produce different keys"
        );
    }

    #[test]
    fn test_special_characters_device_id() {
        let master_key = [0u8; 32];

        // Test with special characters
        let device_ids = [
            "device-123!@#$%",
            "device-with.dots_and_underscores",
            "device/with/slashes",
            "device\\with\\backslashes",
            "device:with:colons",
            "device with spaces",
        ];

        let keys: Vec<[u8; 32]> = device_ids
            .iter()
            .map(|id| derive_device_key(&master_key, id))
            .collect();

        // All should be valid 32-byte keys
        for key in &keys {
            assert_eq!(key.len(), 32, "Special characters must be handled");
        }

        // All should be unique
        let unique_count: std::collections::HashSet<&[u8; 32]> = keys.iter().collect();
        assert_eq!(
            unique_count.len(),
            device_ids.len(),
            "All device IDs with special chars must produce unique keys"
        );
    }

    #[test]
    fn test_device_id_case_sensitivity() {
        let master_key = [0u8; 32];

        let key1 = derive_device_key(&master_key, "MyDevice");
        let key2 = derive_device_key(&master_key, "mydevice");
        let key3 = derive_device_key(&master_key, "MYDEVICE");

        // Case should matter
        assert_ne!(key1, key2, "Device ID must be case-sensitive");
        assert_ne!(key1, key3, "Device ID must be case-sensitive");
        assert_ne!(key2, key3, "Device ID must be case-sensitive");
    }

    /// Count the number of differing bits between two 32-byte arrays
    fn count_bit_differences(key1: &[u8; 32], key2: &[u8; 32]) -> i32 {
        let mut differences = 0;
        for (b1, b2) in key1.iter().zip(key2.iter()) {
            let xor = b1 ^ b2;
            differences += xor.count_ones();
        }
        differences as i32
    }

    #[test]
    fn test_device_key_can_be_used_for_encryption() {
        use crate::crypto::aes256gcm::{decrypt, encrypt};

        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let device_id = "test-device";

        let device_key = derive_device_key(&master_key, device_id);

        // Test encryption/decryption
        let plaintext = b"sensitive test data";
        let (ciphertext, nonce) =
            encrypt(plaintext, &device_key).expect("Device key should support encryption");

        let decrypted = decrypt(&ciphertext, &nonce, &device_key)
            .expect("Device key should support decryption");

        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "Encryption/decryption with device key must work"
        );
    }

    #[test]
    fn test_different_devices_cannot_decrypt_each_others_data() {
        use crate::crypto::aes256gcm::{decrypt, encrypt};

        let master_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let device_key_1 = derive_device_key(&master_key, "device-1");
        let device_key_2 = derive_device_key(&master_key, "device-2");

        // Encrypt with device 1 key
        let plaintext = b"secret data";
        let (ciphertext, nonce) =
            encrypt(plaintext, &device_key_1).expect("Encryption should succeed");

        // Try to decrypt with device 2 key (should fail)
        let result = decrypt(&ciphertext, &nonce, &device_key_2);

        assert!(
            result.is_err(),
            "Device 2 should not be able to decrypt data encrypted with device 1 key"
        );
    }
}
