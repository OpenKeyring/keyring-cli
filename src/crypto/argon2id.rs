use anyhow::Result;
use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;

/// Device capability level for Argon2id parameter selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCapability {
    High,   // 6GB+ RAM, 6+ cores
    Medium, // 3GB+ RAM, 4+ cores
    Low,    // Below medium
}

/// Argon2id parameters
#[derive(Debug, Clone, Copy)]
pub struct Argon2Params {
    pub time: u32,
    pub memory: u32, // MB
    pub parallelism: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self::for_capability(DeviceCapability::Medium)
    }
}

impl Argon2Params {
    /// Get parameters based on device capability
    pub fn for_capability(capability: DeviceCapability) -> Self {
        match capability {
            DeviceCapability::High => Self {
                time: 3,
                memory: 64, // 64 MB
                parallelism: 2,
            },
            DeviceCapability::Medium => Self {
                time: 2,
                memory: 48, // 48 MB
                parallelism: 2,
            },
            DeviceCapability::Low => Self {
                time: 2,
                memory: 32, // 32 MB
                parallelism: 1,
            },
        }
    }
}

/// Detect current device capability
pub fn detect_device_capability() -> DeviceCapability {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let total_memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let cpu_count = sys.cpus().len();

    if total_memory_gb >= 6.0 && cpu_count >= 6 {
        DeviceCapability::High
    } else if total_memory_gb >= 3.0 && cpu_count >= 4 {
        DeviceCapability::Medium
    } else {
        DeviceCapability::Low
    }
}

/// Derive a 256-bit key from password using Argon2id
///
/// # Arguments
/// * `password` - The password to derive from
/// * `salt` - 16-byte salt value
///
/// # Returns
/// 32-byte derived key
pub fn derive_key(password: &str, salt: &[u8; 16]) -> Result<Vec<u8>> {
    let params = Argon2Params::default();

    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(params.memory * 1024, params.time, params.parallelism, None)
            .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {}", e))?,
    );

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Argon2 hashing failed: {}", e))?;

    Ok(key.to_vec())
}

/// Generate a random 16-byte salt
pub fn generate_salt() -> [u8; 16] {
    rand::thread_rng().gen()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_consistent() {
        let password = "test-password";
        let salt = [0u8; 16]; // Use a fixed 16-byte array
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [1u8; 16]; // Use a fixed 16-byte array
        let key1 = derive_key("password1", &salt).unwrap();
        let key2 = derive_key("password2", &salt).unwrap();
        assert_ne!(key1, key2);
    }
}
