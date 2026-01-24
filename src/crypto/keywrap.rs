//! Key wrapping functionality for key hierarchy

use anyhow::Result;

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
        rand::thread_rng().fill(&mut key);
        Ok(DataEncryptionKey(key))
    }

    fn generate_recovery_key() -> Result<RecoveryKey> {
        use rand::Rng;
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        Ok(RecoveryKey(key))
    }

    fn generate_device_key() -> Result<DeviceKey> {
        use rand::Rng;
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        Ok(DeviceKey(key))
    }
}
