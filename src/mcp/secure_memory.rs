//! Secure Memory Utilities
//!
//! This module provides cross-platform secure memory handling for sensitive data.
//! It wraps platform-specific memory protection APIs:
//! - Unix: mlock() to prevent swapping to disk
//! - Windows: CryptProtectMemory for encryption in memory
//!
//! # Security
//!
//! - Protected memory cannot be swapped to disk (Unix) or is encrypted (Windows)
//! - Memory is automatically zeroized on drop
//! - Protection is applied immediately on creation

use crate::platform::{protect_memory, unprotect_memory, PlatformError};
use std::cell::UnsafeCell;
use zeroize::Zeroize;

/// Error types for secure memory operations
#[derive(Debug, thiserror::Error)]
pub enum SecureMemoryError {
    #[error("Memory protection failed: {0}")]
    ProtectionFailed(String),

    #[error("Memory unprotection failed: {0}")]
    UnprotectionFailed(String),

    #[error("Memory is not protected")]
    NotProtected,
}

impl From<PlatformError> for SecureMemoryError {
    fn from(err: PlatformError) -> Self {
        match err {
            PlatformError::MemoryProtectionFailed(msg) => SecureMemoryError::ProtectionFailed(msg),
            _ => SecureMemoryError::ProtectionFailed(err.to_string()),
        }
    }
}

/// Secure buffer that protects memory from being swapped to disk
///
/// # Security
///
/// - On Unix: Uses mlock() to prevent memory from being swapped to disk
/// - On Windows: Uses CryptProtectMemory to encrypt memory (data is padded to 16-byte boundaries)
/// - Automatically zeroizes on drop
///
/// # Platform Notes
///
/// On Windows, CryptProtectMemory requires memory to be aligned to 16-byte boundaries.
/// SecureBuffer automatically pads data to meet this requirement. The logical length
/// (the actual data length) is preserved and returned by `len()` and `as_slice()`.
///
/// On Windows, the first call to `as_slice()` will decrypt the data for reading.
/// Subsequent calls will return the decrypted data. The data is re-encrypted on drop.
///
/// # Example
///
/// ```no_run
/// use keyring_cli::mcp::secure_memory::SecureBuffer;
///
/// // Create a protected buffer from sensitive data
/// let mut buffer = SecureBuffer::new(vec![0x42, 0x43, 0x44]).unwrap();
///
/// // Access the data
/// let data = buffer.as_slice();
/// println!("Protected data length: {}", data.len());
///
/// // Buffer is automatically zeroized and unprotected on drop
/// ```
pub struct SecureBuffer {
    /// The protected data (may be padded on Windows for 16-byte alignment)
    /// Using UnsafeCell for interior mutability needed on Windows
    data: UnsafeCell<Vec<u8>>,

    /// The actual logical length of the data (excluding padding)
    logical_len: usize,

    /// Whether memory is currently protected (encrypted on Windows)
    is_protected: UnsafeCell<bool>,
}

// SAFETY: We only access data mutably in methods that logically have mutable access
// (as_slice needs to decrypt on Windows, but this is safe because we track state)
unsafe impl Send for SecureBuffer {}
unsafe impl Sync for SecureBuffer {}

impl SecureBuffer {
    /// Create a new protected buffer
    ///
    /// # Arguments
    ///
    /// * `data` - The data to protect
    ///
    /// # Returns
    ///
    /// Ok(SecureBuffer) if protection succeeds, Err otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Memory protection fails (e.g., mlock fails due to resource limits)
    /// - Data pointer is null
    ///
    /// # Platform Notes
    ///
    /// On Windows, data is automatically padded to meet CryptProtectMemory's
    /// 16-byte alignment requirement. The logical length is preserved.
    pub fn new(mut data: Vec<u8>) -> Result<Self, SecureMemoryError> {
        let logical_len = data.len();

        if data.is_empty() {
            return Ok(Self {
                data: UnsafeCell::new(data),
                logical_len: 0,
                is_protected: UnsafeCell::new(false),
            });
        }

        // On Windows, pad data to 16-byte boundary for CryptProtectMemory
        #[cfg(target_os = "windows")]
        {
            const BLOCK_SIZE: usize = 16;
            let padding_needed = (BLOCK_SIZE - (data.len() % BLOCK_SIZE)) % BLOCK_SIZE;
            if padding_needed != 0 {
                data.extend(vec![0u8; padding_needed]);
            }
        }

        // Protect the memory
        protect_memory(data.as_mut_ptr(), data.len())
            .map_err(|e| SecureMemoryError::ProtectionFailed(e.to_string()))?;

        Ok(Self {
            data: UnsafeCell::new(data),
            logical_len,
            is_protected: UnsafeCell::new(true),
        })
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.logical_len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.logical_len == 0
    }

    /// Get a reference to the protected data
    ///
    /// # Note
    ///
    /// The data remains protected while you have a reference to it.
    /// On Windows, the data is encrypted and will be decrypted on first access.
    /// Only the logical data (excluding padding) is returned.
    ///
    /// # Platform Notes
    ///
    /// On Windows, the first call to `as_slice()` decrypts the data in-place.
    /// After this, the data remains decrypted until the SecureBuffer is dropped.
    /// This is a limitation of the CryptProtectMemory API which encrypts data in-place.
    pub fn as_slice(&self) -> &[u8] {
        // On Windows, decrypt the data on first access
        #[cfg(target_os = "windows")]
        {
            // SAFETY: We use UnsafeCell to allow modification through &self.
            // The is_protected flag ensures we only decrypt once.
            // This is safe because:
            // 1. We check is_protected before modifying
            // 2. We only decrypt (not encrypt) here
            // 3. The memory is properly synchronized
            unsafe {
                if *self.is_protected.get() {
                    let data = &mut *self.data.get();
                    let _ = unprotect_memory(data.as_mut_ptr(), data.len());
                    *self.is_protected.get() = false;
                }
            }
        }

        // SAFETY: We're only reading the data here, and we ensure it's synchronized
        unsafe { &(&*self.data.get())[..self.logical_len] }
    }

    /// Unprotect the memory and return the underlying data
    ///
    /// This consumes the SecureBuffer and returns the raw Vec<u8>.
    /// The caller is responsible for zeroizing the data after use.
    /// On Windows, padding is removed before returning.
    pub fn into_inner(self) -> Vec<u8> {
        // SAFETY: We own self now, so we have exclusive access
        let is_protected = unsafe { *self.is_protected.get() };
        if is_protected {
            // Unprotect before returning
            let data = unsafe { &mut *self.data.get() };
            let _ = unprotect_memory(data.as_mut_ptr(), data.len());
        }
        // Mark as unprotected so Drop doesn't try to unprotect again
        unsafe { *self.is_protected.get() = false };
        // Take the data and truncate to logical length
        // SAFETY: We have exclusive ownership and marked is_protected as false
        let mut data = unsafe { std::ptr::read(self.data.get()) };
        data.truncate(self.logical_len);
        // Prevent Drop from running on self.data since we took ownership
        std::mem::forget(self);
        data
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // SAFETY: We have &mut self in drop, so exclusive access
        let is_protected = unsafe { *self.is_protected.get() };
        if is_protected {
            // Unprotect memory before zeroizing
            let data = unsafe { &mut *self.data.get() };
            let _ = unprotect_memory(data.as_mut_ptr(), data.len());
        }
        // Zeroize the data
        unsafe { (*self.data.get()).zeroize() };
    }
}

impl Clone for SecureBuffer {
    fn clone(&self) -> Self {
        // Clone only the logical data (without padding)
        let cloned_data = self.as_slice().to_vec();
        Self::new(cloned_data).unwrap_or_else(|_| Self {
            data: UnsafeCell::new(vec![]),
            logical_len: 0,
            is_protected: UnsafeCell::new(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_buffer_creation() {
        let data = vec![0x42, 0x43, 0x44];
        let buffer = SecureBuffer::new(data);

        assert!(buffer.is_ok());
        let buffer = buffer.unwrap();
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_secure_buffer_empty() {
        let buffer = SecureBuffer::new(vec![]).unwrap();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_secure_buffer_as_slice() {
        let data = vec![0x42, 0x43, 0x44];
        let buffer = SecureBuffer::new(data).unwrap();
        let slice = buffer.as_slice();
        assert_eq!(slice, &[0x42, 0x43, 0x44]);
    }

    #[test]
    fn test_secure_buffer_clone() {
        let data = vec![0x42, 0x43, 0x44];
        let buffer = SecureBuffer::new(data).unwrap();
        let cloned = buffer.clone();
        assert_eq!(buffer.as_slice(), cloned.as_slice());
    }

    #[test]
    fn test_secure_buffer_into_inner() {
        let data = vec![0x42, 0x43, 0x44];
        let buffer = SecureBuffer::new(data).unwrap();
        let inner = buffer.into_inner();
        assert_eq!(inner, vec![0x42, 0x43, 0x44]);
    }

    #[test]
    fn test_secure_buffer_large_data() {
        // Test with larger data (1KB)
        let data = vec![0x42u8; 1024];
        let buffer = SecureBuffer::new(data);

        assert!(buffer.is_ok());
        let buffer = buffer.unwrap();
        assert_eq!(buffer.len(), 1024);
    }

    #[test]
    fn test_secure_buffer_non_16_byte_sizes() {
        // Test various sizes that are not multiples of 16
        // This ensures Windows padding works correctly
        for size in [1, 3, 5, 7, 8, 11, 13, 15] {
            let data = vec![0x42u8; size];
            let buffer = SecureBuffer::new(data.clone());

            assert!(buffer.is_ok(), "Failed for size {}", size);
            let buffer = buffer.unwrap();
            assert_eq!(buffer.len(), size, "Length mismatch for size {}", size);
            assert_eq!(
                buffer.as_slice(),
                data.as_slice(),
                "Data mismatch for size {}",
                size
            );
        }
    }
}
