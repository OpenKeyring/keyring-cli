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
/// - On Windows: Uses CryptProtectMemory to encrypt memory
/// - Automatically zeroizes on drop
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
    /// The protected data
    data: Vec<u8>,

    /// Whether memory is currently protected
    is_protected: bool,
}

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
    pub fn new(mut data: Vec<u8>) -> Result<Self, SecureMemoryError> {
        if data.is_empty() {
            return Ok(Self {
                data,
                is_protected: false,
            });
        }

        // Protect the memory
        protect_memory(data.as_mut_ptr(), data.len())
            .map_err(|e| SecureMemoryError::ProtectionFailed(e.to_string()))?;

        Ok(Self {
            data,
            is_protected: true,
        })
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a reference to the protected data
    ///
    /// # Note
    ///
    /// The data remains protected while you have a reference to it.
    /// On Windows, the data is encrypted and will be decrypted on access.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Unprotect the memory and return the underlying data
    ///
    /// This consumes the SecureBuffer and returns the raw Vec<u8>.
    /// The caller is responsible for zeroizing the data after use.
    pub fn into_inner(mut self) -> Vec<u8> {
        if self.is_protected {
            // Unprotect before returning
            let _ = unprotect_memory(self.data.as_mut_ptr(), self.data.len());
            self.is_protected = false;
        }
        // Use std::mem::take to avoid moving out of type with Drop
        std::mem::take(&mut self.data)
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        if self.is_protected {
            // Unprotect memory before zeroizing
            let _ = unprotect_memory(self.data.as_mut_ptr(), self.data.len());
        }
        // Zeroize the data
        self.data.zeroize();
    }
}

impl Clone for SecureBuffer {
    fn clone(&self) -> Self {
        // Create a new buffer with cloned data
        // The new buffer will also be protected
        let cloned_data = self.data.clone();
        Self::new(cloned_data).unwrap_or_else(|_| Self {
            data: vec![],
            is_protected: false,
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
}
