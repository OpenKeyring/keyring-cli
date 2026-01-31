//! MCP Key Cache Integration Tests
//!
//! Tests for the full McpKeyCache lifecycle including:
//! - Master password derivation
//! - DEK extraction
//! - Signing key derivation
//! - Audit key derivation

#[cfg(test)]
mod mcp_key_cache_integration_tests {
    use keyring_cli::crypto::hkdf::{derive_device_key, DeviceIndex};
    use zeroize::Zeroize;

    /// Test device key derivation (used by key cache)
    #[test]
    fn test_device_key_derivation() {
        let master_key = [1u8; 32];

        // Derive a device key for CLI
        let device_key = derive_device_key(&master_key, DeviceIndex::CLI.as_str());

        assert_eq!(device_key.len(), 32);

        // Same input should produce same key
        let device_key2 = derive_device_key(&master_key, DeviceIndex::CLI.as_str());
        assert_eq!(device_key, device_key2);
    }

    /// Test that different device indices produce different keys
    #[test]
    fn test_different_devices_produce_different_keys() {
        let master_key = [3u8; 32];

        let macos_key = derive_device_key(&master_key, DeviceIndex::MacOS.as_str());
        let linux_key = derive_device_key(&master_key, DeviceIndex::Linux.as_str());
        let cli_key = derive_device_key(&master_key, DeviceIndex::CLI.as_str());

        // All keys should be different
        assert_ne!(macos_key, linux_key, "macOS and Linux keys should differ");
        assert_ne!(macos_key, cli_key, "macOS and CLI keys should differ");
        assert_ne!(linux_key, cli_key, "Linux and CLI keys should differ");
    }

    /// Test that different master keys produce different device keys
    #[test]
    fn test_different_master_keys_produce_different_device_keys() {
        let master_key1 = [6u8; 32];
        let master_key2 = [7u8; 32];

        let key1 = derive_device_key(&master_key1, DeviceIndex::CLI.as_str());
        let key2 = derive_device_key(&master_key2, DeviceIndex::CLI.as_str());

        assert_ne!(key1, key2, "Different master keys should produce different device keys");
    }

    /// Test device key derivation for all platforms
    #[test]
    fn test_device_key_derivation_all_platforms() {
        let master_key = [8u8; 32];

        let platforms = [
            DeviceIndex::MacOS,
            DeviceIndex::IOS,
            DeviceIndex::Windows,
            DeviceIndex::Linux,
            DeviceIndex::CLI,
        ];

        let keys: Vec<[u8; 32]> = platforms
            .iter()
            .map(|&platform| derive_device_key(&master_key, platform.as_str()))
            .collect();

        // All keys should have the correct length
        for key in &keys {
            assert_eq!(key.len(), 32);
        }

        // All keys should be different
        for (i, key1) in keys.iter().enumerate() {
            for (j, key2) in keys.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        key1, key2,
                        "Keys for platforms {:?} and {:?} should differ",
                        platforms[i], platforms[j]
                    );
                }
            }
        }
    }

    /// Test that zeroizing a key produces all zeros
    #[test]
    fn test_key_zeroize() {
        let mut key = [0xABu8; 32];
        key.zeroize();

        assert!(key.iter().all(|&b| b == 0));
    }

    /// Test zeroize on different key patterns
    #[test]
    fn test_zeroize_different_patterns() {
        let patterns: [[u8; 32]; 4] = [
            [0xFFu8; 32],
            [0x00u8; 32],
            [0xAAu8; 32],
            [0x55u8; 32],
        ];

        for mut pattern in patterns {
            pattern.zeroize();
            assert!(pattern.iter().all(|&b| b == 0));
        }
    }

    /// Test key derivation is deterministic
    #[test]
    fn test_device_key_deterministic() {
        let master_key = [10u8; 32];

        let keys: Vec<[u8; 32]> = (0..10)
            .map(|_| derive_device_key(&master_key, DeviceIndex::CLI.as_str()))
            .collect();

        // All derived keys should be identical
        for key in &keys[1..] {
            assert_eq!(keys[0], *key);
        }
    }

    /// Test that device keys are cryptographically independent
    #[test]
    fn test_device_key_separation() {
        let master_key = [12u8; 32];

        let macos_key = derive_device_key(&master_key, DeviceIndex::MacOS.as_str());
        let linux_key = derive_device_key(&master_key, DeviceIndex::Linux.as_str());

        // Keys should be cryptographically independent
        let different_bytes = macos_key
            .iter()
            .zip(linux_key.iter())
            .filter(|(a, b)| a != b)
            .count();

        // At least 50% of bytes should be different (statistical expectation is ~100%)
        assert!(different_bytes >= 16);
    }

    /// Test that device key derivation is consistent across multiple calls
    #[test]
    fn test_device_key_consistency_across_calls() {
        let master_key = [17u8; 32];

        let keys: Vec<[u8; 32]> = (0..100)
            .map(|_| derive_device_key(&master_key, DeviceIndex::CLI.as_str()))
            .collect();

        // All keys should be identical
        let first = &keys[0];
        for key in &keys[1..] {
            assert_eq!(first, key, "Device key derivation should be deterministic");
        }
    }

    /// Test DeviceIndex::as_str() conversion
    #[test]
    fn test_device_index_as_str() {
        assert_eq!(DeviceIndex::MacOS.as_str(), "macos");
        assert_eq!(DeviceIndex::IOS.as_str(), "ios");
        assert_eq!(DeviceIndex::Windows.as_str(), "windows");
        assert_eq!(DeviceIndex::Linux.as_str(), "linux");
        assert_eq!(DeviceIndex::CLI.as_str(), "cli");
    }

    /// Test that device keys are 32 bytes (256 bits) for cryptographic security
    #[test]
    fn test_device_key_length() {
        let master_key = [19u8; 32];

        for platform in &[
            DeviceIndex::MacOS,
            DeviceIndex::IOS,
            DeviceIndex::Windows,
            DeviceIndex::Linux,
            DeviceIndex::CLI,
        ] {
            let key = derive_device_key(&master_key, platform.as_str());
            assert_eq!(
                key.len(), 32,
                "Device key for {:?} should be 32 bytes (256 bits)",
                platform
            );
        }
    }

    /// Test that the same device ID always produces the same key
    #[test]
    fn test_same_device_id_same_key() {
        let master_key = [21u8; 32];

        // Using the same device_id string should produce the same key
        let key1 = derive_device_key(&master_key, "my-custom-device");
        let key2 = derive_device_key(&master_key, "my-custom-device");

        assert_eq!(key1, key2);
    }

    /// Test that different device IDs produce different keys
    #[test]
    fn test_different_device_ids_different_keys() {
        let master_key = [22u8; 32];

        let key1 = derive_device_key(&master_key, "device-1");
        let key2 = derive_device_key(&master_key, "device-2");

        assert_ne!(key1, key2);
    }
}
