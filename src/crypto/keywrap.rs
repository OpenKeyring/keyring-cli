//! Key wrapping functionality for key hierarchy

use crate::crypto::aes256gcm;
use anyhow::Result;

/// Wrap a key using AES-256-GCM
/// Returns: (encrypted_key, nonce)
pub fn wrap_key(key_to_wrap: &[u8; 32], wrapping_key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12])> {
    aes256gcm::encrypt(key_to_wrap, wrapping_key)
}

/// Unwrap a key using AES-256-GCM
pub fn unwrap_key(
    wrapped_key: &[u8],
    nonce: &[u8; 12],
    wrapping_key: &[u8; 32],
) -> Result<[u8; 32]> {
    let unwrapped = aes256gcm::decrypt(wrapped_key, nonce, wrapping_key)?;

    let mut key = [0u8; 32];
    key.copy_from_slice(unwrapped.as_slice());
    Ok(key)
}

/// Master key derived from user's master password
pub struct MasterKey(pub [u8; 32]);

/// Data encryption key for actual record encryption
pub struct DataEncryptionKey(pub [u8; 32]);

/// Recovery key (BIP39 mnemonic) for emergency access
pub struct RecoveryKey(pub [u8; 32]);

/// Device-specific key for biometric unlock
pub struct DeviceKey(pub [u8; 32]);

/// Key hierarchy containing all wrapped keys
pub struct KeyHierarchy {
    pub master_key: MasterKey,
    pub dek: DataEncryptionKey,
    pub recovery_key: RecoveryKey,
    pub device_key: DeviceKey,
}

impl KeyHierarchy {
    /// Setup new key hierarchy (first-time initialization)
    pub fn setup(master_password: &str) -> Result<Self> {
        // Generate random keys
        let dek = Self::generate_dek()?;
        let recovery_key = Self::generate_recovery_key()?;
        let device_key = Self::generate_device_key()?;

        // Derive master key from password
        let master_key = Self::derive_master_key(master_password)?;

        // Wrap keys with master key (TODO: implement wrapping)

        Ok(Self {
            master_key,
            dek,
            recovery_key,
            device_key,
        })
    }

    /// Unlock existing key hierarchy
    pub fn unlock(_master_password: &str, _wrapped_keys_path: &std::path::Path) -> Result<Self> {
        // TODO: Implement unlocking from wrapped keys
        anyhow::bail!("KeyHierarchy::unlock not yet implemented")
    }

    fn derive_master_key(password: &str) -> Result<MasterKey> {
        use super::argon2id;
        let salt = super::argon2id::generate_salt();
        let key_bytes = argon2id::derive_key(password, &salt)?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(MasterKey(key))
    }

    fn generate_dek() -> Result<DataEncryptionKey> {
        use rand::Rng;
        let mut key = [0u8; 32];
        rand::rng().fill(&mut key);
        Ok(DataEncryptionKey(key))
    }

    fn generate_recovery_key() -> Result<RecoveryKey> {
        use rand::Rng;
        let mut key = [0u8; 32];
        rand::rng().fill(&mut key);
        Ok(RecoveryKey(key))
    }

    fn generate_device_key() -> Result<DeviceKey> {
        use rand::Rng;
        let mut key = [0u8; 32];
        rand::rng().fill(&mut key);
        Ok(DeviceKey(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_key() {
        let wrapping_key = [1u8; 32];
        let key_to_wrap = [2u8; 32];

        let wrapped = wrap_key(&key_to_wrap, &wrapping_key).unwrap();
        let unwrapped = unwrap_key(&wrapped.0, &wrapped.1, &wrapping_key).unwrap();

        assert_eq!(key_to_wrap.to_vec(), unwrapped.to_vec());
    }

    #[test]
    fn test_wrapped_key_different_with_different_nonce() {
        let wrapping_key = [1u8; 32];
        let key_to_wrap = [2u8; 32];

        let wrapped1 = wrap_key(&key_to_wrap, &wrapping_key).unwrap();
        let wrapped2 = wrap_key(&key_to_wrap, &wrapping_key).unwrap();

        // Different nonces should produce different ciphertexts
        assert_ne!(wrapped1.0, wrapped2.0);
    }

    #[test]
    fn test_unwrap_with_wrong_key_fails() {
        let wrapping_key = [1u8; 32];
        let wrong_key = [99u8; 32];
        let key_to_wrap = [2u8; 32];

        let wrapped = wrap_key(&key_to_wrap, &wrapping_key).unwrap();
        let result = unwrap_key(&wrapped.0, &wrapped.1, &wrong_key);

        assert!(result.is_err());
    }
}
