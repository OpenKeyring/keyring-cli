//! Key wrapping functionality for key hierarchy

use crate::crypto::aes256gcm;
use crate::types::SensitiveString;
use anyhow::Result;
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

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

/// Wrapped key with encrypted data and nonce
#[derive(Clone, Debug)]
pub struct WrappedKey {
    pub wrapped_data: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Drop for WrappedKey {
    fn drop(&mut self) {
        self.wrapped_data.zeroize();
        self.nonce.zeroize();
    }
}

/// Key hierarchy containing all wrapped keys
pub struct KeyHierarchy {
    pub master_key: MasterKey,
    pub dek: DataEncryptionKey,
    pub recovery_key: RecoveryKey,
    pub device_key: DeviceKey,
    /// Salt used for key derivation (stored for consistency)
    salt: [u8; 16],
}

impl KeyHierarchy {
    /// Setup new key hierarchy (first-time initialization) with SensitiveString
    pub fn setup_sensitive(master_password: &SensitiveString<String>) -> Result<Self> {
        use super::argon2id;

        // Generate salt for key derivation
        let salt = argon2id::generate_salt();

        // Generate random keys
        let dek = Self::generate_dek()?;
        let recovery_key = Self::generate_recovery_key()?;
        let device_key = Self::generate_device_key()?;

        // Derive master key from password with salt
        let key_bytes = argon2id::derive_key_sensitive(master_password, &salt)?;
        let mut master_key_array = [0u8; 32];
        master_key_array.copy_from_slice(&key_bytes);
        let master_key = MasterKey(master_key_array);

        Ok(Self {
            master_key,
            dek,
            recovery_key,
            device_key,
            salt,
        })
    }

    /// Setup new key hierarchy (first-time initialization)
    pub fn setup(master_password: &str) -> Result<Self> {
        use super::argon2id;

        // Generate salt for key derivation
        let salt = argon2id::generate_salt();

        // Generate random keys
        let dek = Self::generate_dek()?;
        let recovery_key = Self::generate_recovery_key()?;
        let device_key = Self::generate_device_key()?;

        // Derive master key from password with salt
        let key_bytes = argon2id::derive_key(master_password, &salt)?;
        let mut master_key_array = [0u8; 32];
        master_key_array.copy_from_slice(&key_bytes);
        let master_key = MasterKey(master_key_array);

        Ok(Self {
            master_key,
            dek,
            recovery_key,
            device_key,
            salt,
        })
    }

    /// Unlock existing key hierarchy with SensitiveString
    pub fn unlock_sensitive(
        wrapped_keys_path: &Path,
        master_password: &SensitiveString<String>,
    ) -> Result<Self> {
        use super::argon2id;

        // Load salt from file
        let salt_bytes = fs::read(wrapped_keys_path.join("salt"))?;
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&salt_bytes[..16]);

        // Derive master key from password with stored salt
        let key_bytes = argon2id::derive_key_sensitive(master_password, &salt)?;
        let mut master_key_array = [0u8; 32];
        master_key_array.copy_from_slice(&key_bytes);
        let master_key = MasterKey(master_key_array);

        // Load wrapped DEK
        let wrapped_dek = fs::read(wrapped_keys_path.join("wrapped_dek"))?;
        let nonce_dek: [u8; 12] = wrapped_dek[0..12].try_into().unwrap();
        let dek_bytes = &wrapped_dek[12..];
        let dek = Self::unwrap_key(dek_bytes, &nonce_dek, &master_key.0)?;

        // Load wrapped RecoveryKey
        let wrapped_rec = fs::read(wrapped_keys_path.join("wrapped_recovery"))?;
        let nonce_rec: [u8; 12] = wrapped_rec[0..12].try_into().unwrap();
        let rec_bytes = &wrapped_rec[12..];
        let recovery_key = Self::unwrap_key(rec_bytes, &nonce_rec, &master_key.0)?;

        // Load wrapped DeviceKey
        let wrapped_dev = fs::read(wrapped_keys_path.join("wrapped_device"))?;
        let nonce_dev: [u8; 12] = wrapped_dev[0..12].try_into().unwrap();
        let dev_bytes = &wrapped_dev[12..];
        let device_key = Self::unwrap_key(dev_bytes, &nonce_dev, &master_key.0)?;

        Ok(Self {
            master_key,
            dek: DataEncryptionKey(dek),
            recovery_key: RecoveryKey(recovery_key),
            device_key: DeviceKey(device_key),
            salt,
        })
    }

    /// Unlock existing key hierarchy
    pub fn unlock(wrapped_keys_path: &Path, master_password: &str) -> Result<Self> {
        use super::argon2id;

        // Load salt from file
        let salt_bytes = fs::read(wrapped_keys_path.join("salt"))?;
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&salt_bytes[..16]);

        // Derive master key from password with stored salt
        let key_bytes = argon2id::derive_key(master_password, &salt)?;
        let mut master_key_array = [0u8; 32];
        master_key_array.copy_from_slice(&key_bytes);
        let master_key = MasterKey(master_key_array);

        // Load wrapped DEK
        let wrapped_dek = fs::read(wrapped_keys_path.join("wrapped_dek"))?;
        let nonce_dek: [u8; 12] = wrapped_dek[0..12].try_into().unwrap();
        let dek_bytes = &wrapped_dek[12..];
        let dek = Self::unwrap_key(dek_bytes, &nonce_dek, &master_key.0)?;

        // Load wrapped RecoveryKey
        let wrapped_rec = fs::read(wrapped_keys_path.join("wrapped_recovery"))?;
        let nonce_rec: [u8; 12] = wrapped_rec[0..12].try_into().unwrap();
        let rec_bytes = &wrapped_rec[12..];
        let recovery_key = Self::unwrap_key(rec_bytes, &nonce_rec, &master_key.0)?;

        // Load wrapped DeviceKey
        let wrapped_dev = fs::read(wrapped_keys_path.join("wrapped_device"))?;
        let nonce_dev: [u8; 12] = wrapped_dev[0..12].try_into().unwrap();
        let dev_bytes = &wrapped_dev[12..];
        let device_key = Self::unwrap_key(dev_bytes, &nonce_dev, &master_key.0)?;

        Ok(Self {
            master_key,
            dek: DataEncryptionKey(dek),
            recovery_key: RecoveryKey(recovery_key),
            device_key: DeviceKey(device_key),
            salt,
        })
    }

    /// Save wrapped keys to directory
    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;

        // Save salt
        fs::write(dir.join("salt"), self.salt)?;

        // Wrap and save DEK
        let (wrapped_dek_bytes, nonce_dek) = self.wrap_key(&self.dek.0, &self.master_key.0)?;
        let mut dek_file = nonce_dek.to_vec();
        dek_file.extend_from_slice(&wrapped_dek_bytes);
        fs::write(dir.join("wrapped_dek"), dek_file)?;

        // Wrap and save RecoveryKey
        let (wrapped_rec_bytes, nonce_rec) =
            self.wrap_key(&self.recovery_key.0, &self.master_key.0)?;
        let mut rec_file = nonce_rec.to_vec();
        rec_file.extend_from_slice(&wrapped_rec_bytes);
        fs::write(dir.join("wrapped_recovery"), rec_file)?;

        // Wrap and save DeviceKey
        let (wrapped_dev_bytes, nonce_dev) =
            self.wrap_key(&self.device_key.0, &self.master_key.0)?;
        let mut dev_file = nonce_dev.to_vec();
        dev_file.extend_from_slice(&wrapped_dev_bytes);
        fs::write(dir.join("wrapped_device"), dev_file)?;

        Ok(())
    }

    /// Wrap a key using SensitiveString
    pub fn wrap_key_sensitive(&self, key: &SensitiveString<Vec<u8>>) -> Result<WrappedKey> {
        let key_bytes = key.get();
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Key must be 32 bytes, got {}",
                key_bytes.len()
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(key_bytes);

        let (wrapped_data, nonce) = self.wrap_key(&key_array, &self.master_key.0)?;

        Ok(WrappedKey {
            wrapped_data,
            nonce: nonce.to_vec(),
        })
    }

    /// Unwrap a key returning SensitiveString
    pub fn unwrap_key_sensitive(&self, wrapped: &WrappedKey) -> Result<SensitiveString<Vec<u8>>> {
        let nonce_array: [u8; 12] = wrapped
            .nonce
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

        let unwrapped = Self::unwrap_key(&wrapped.wrapped_data, &nonce_array, &self.master_key.0)?;
        Ok(SensitiveString::new(unwrapped.to_vec()))
    }

    /// Wrap a key using the master key
    fn wrap_key(&self, key: &[u8; 32], wrapping_key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12])> {
        super::wrap_key(key, wrapping_key)
    }

    /// Unwrap a key using the master key
    fn unwrap_key(wrapped: &[u8], nonce: &[u8; 12], wrapping_key: &[u8; 32]) -> Result<[u8; 32]> {
        super::unwrap_key(wrapped, nonce, wrapping_key)
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
