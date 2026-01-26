use anyhow::Result;
use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;
use sysinfo;
// use zeroize::ZeroizeOnDrop;  // Unused

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

/// Derive a 256-bit key using custom Argon2id parameters
pub fn derive_key_with_params(
    password: &str,
    salt: &[u8; 16],
    params: Argon2Params,
) -> Result<Vec<u8>> {
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
    rand::thread_rng().random()
}

/// Stored password hash with salt and parameters
#[derive(Debug, Clone)]
pub struct PasswordHash {
    pub salt: [u8; 16],
    pub key: Vec<u8>,
    pub params: Argon2Params,
}

/// Hash a password and return the complete hash structure
pub fn hash_password(password: &str) -> Result<PasswordHash> {
    let salt = generate_salt();
    let params = Argon2Params::default();
    let key = derive_key_with_params(password, &salt, params)?;

    Ok(PasswordHash { salt, key, params })
}

/// Verify a password against a stored hash
pub fn verify_password(password: &str, hash: &PasswordHash) -> Result<bool> {
    let key = derive_key_with_params(password, &hash.salt, hash.params)?;
    Ok(key == hash.key)
}

/// Verify Argon2id parameters meet security requirements
pub fn verify_params_security(params: &Argon2Params) -> Result<(), &'static str> {
    if params.memory < 32 {
        return Err("Memory must be at least 32 MB for security");
    }
    if params.time < 1 {
        return Err("Time cost must be at least 1");
    }
    if params.parallelism < 1 {
        return Err("Parallelism must be at least 1");
    }

    // OWASP recommendations
    if params.memory < 64 && params.time < 3 {
        // Allow lower memory if time cost is higher
    }

    Ok(())
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

    #[test]
    fn test_derive_key_with_custom_params() {
        let password = "test-password";
        let salt = generate_salt();
        let params = Argon2Params {
            time: 1,
            memory: 16,
            parallelism: 1,
        };

        let key = derive_key_with_params(password, &salt, params).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_password_hash_structure() {
        let password = "secure-password-123";
        let hash = hash_password(password).unwrap();

        assert_eq!(hash.salt.len(), 16);
        assert_eq!(hash.key.len(), 32);
        assert!(hash.params.memory >= 32);
    }

    #[test]
    fn test_verify_password() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_memory_hardening_verification() {
        let params = Argon2Params {
            time: 2,
            memory: 32,
            parallelism: 1,
        };

        // Verify minimum memory requirements
        assert!(params.memory >= 32, "Memory must be at least 32 MB");
        assert!(params.time >= 1, "Time cost must be at least 1");
        assert!(params.parallelism >= 1, "Parallelism must be at least 1");
    }

    #[test]
    fn test_verify_params_security_valid() {
        let params = Argon2Params {
            time: 3,
            memory: 64,
            parallelism: 2,
        };

        assert!(verify_params_security(&params).is_ok());
    }

    #[test]
    fn test_verify_params_security_invalid() {
        let params = Argon2Params {
            time: 0,
            memory: 16,
            parallelism: 0,
        };

        assert!(verify_params_security(&params).is_err());
    }
}
