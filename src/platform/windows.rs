//! Windows-specific platform functionality
//!
//! Implements memory protection using CryptProtectMemory API.

use crate::error::Result;
use crate::platform::PlatformError;
use std::ptr;
use windows_sys::Win32::Security::Cryptography::*;

/// Protect memory in the current process
///
/// This function encrypts memory in the current process to prevent
/// it from being swapped to disk or read by other processes.
///
/// # Arguments
/// * `addr` - Pointer to the memory region to protect
/// * `len` - Length of the memory region in bytes
///
/// # Returns
/// * `Ok(())` if memory was successfully protected
/// * `Err(PlatformError)` if protection failed
///
/// # Safety
/// The caller must ensure that the memory region is valid and accessible.
///
/// # Platform Notes
/// CryptProtectMemory works on CRYPTPROTECTMEMORY_BLOCK_SIZE (16 bytes) boundaries.
/// The length must be a multiple of 16 bytes.
pub fn protect_memory(addr: *mut u8, len: usize) -> Result<()> {
    if addr.is_null() || len == 0 {
        return Err(PlatformError::MemoryProtectionFailed(
            "Invalid address or length".to_string(),
        )
        .into());
    }

    // CryptProtectMemory requires length to be a multiple of CRYPTPROTECTMEMORY_BLOCK_SIZE
    const BLOCK_SIZE: usize = 16;
    if len % BLOCK_SIZE != 0 {
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "Length must be a multiple of {} bytes (got {})",
            BLOCK_SIZE, len
        ))
        .into());
    }

    // Call CryptProtectMemory
    // dwFlags: 0 = CRYPTPROTECTMEMORY_SAME_PROCESS (only accessible in same process)
    let result = unsafe { CryptProtectMemory(addr as *mut u8, len, 0) };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "CryptProtectMemory failed with error code: {}",
            error_code
        ))
        .into());
    }

    Ok(())
}

/// Unprotect (decrypt) memory in the current process
///
/// This function decrypts memory that was previously protected with CryptProtectMemory.
///
/// # Arguments
/// * `addr` - Pointer to the memory region to unprotect
/// * `len` - Length of the memory region in bytes
///
/// # Returns
/// * `Ok(())` if memory was successfully unprotected
/// * `Err(PlatformError)` if unprotection failed
///
/// # Safety
/// The caller must ensure that the memory region was previously protected.
pub fn unprotect_memory(addr: *mut u8, len: usize) -> Result<()> {
    if addr.is_null() || len == 0 {
        return Err(PlatformError::MemoryProtectionFailed(
            "Invalid address or length".to_string(),
        )
        .into());
    }

    const BLOCK_SIZE: usize = 16;
    if len % BLOCK_SIZE != 0 {
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "Length must be a multiple of {} bytes (got {})",
            BLOCK_SIZE, len
        ))
        .into());
    }

    // Call CryptUnprotectMemory
    let result = unsafe { CryptUnprotectMemory(addr as *mut u8, len, 0) };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "CryptUnprotectMemory failed with error code: {}",
            error_code
        ))
        .into());
    }

    Ok(())
}

/// Get the system memory allocation granularity
///
/// Windows memory allocations are typically aligned to 64KB boundaries.
pub fn allocation_granularity() -> usize {
    unsafe {
        let mut info = std::mem::zeroed::<SYSTEM_INFO>();
        GetSystemInfo(&mut info);
        info.dwAllocationGranularity as usize
    }
}

/// Get the system page size
pub fn page_size() -> usize {
    unsafe {
        let mut info = std::mem::zeroed::<SYSTEM_INFO>();
        GetSystemInfo(&mut info);
        info.dwPageSize as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_memory_aligned() {
        // CryptProtectMemory requires length to be a multiple of 16 bytes
        let mut data = vec![0u8; 32]; // 32 = 2 * 16
        let result = protect_memory(data.as_mut_ptr(), data.len());
        assert!(result.is_ok(), "CryptProtectMemory should succeed for aligned size");

        // Verify the data is actually encrypted (should have changed)
        // Note: We can't decrypt without the original, but we can call unprotect_memory
        let unprotect_result = unprotect_memory(data.as_mut_ptr(), data.len());
        assert!(unprotect_result.is_ok(), "CryptUnprotectMemory should succeed");
    }

    #[test]
    fn test_protect_memory_invalid_length() {
        let mut data = vec![0u8; 15]; // Not a multiple of 16
        let result = protect_memory(data.as_mut_ptr(), data.len());
        assert!(
            result.is_err(),
            "CryptProtectMemory should fail with invalid length"
        );
    }

    #[test]
    fn test_protect_memory_null_pointer() {
        let result = protect_memory(std::ptr::null_mut(), 32);
        assert!(result.is_err(), "CryptProtectMemory should fail with null pointer");
    }

    #[test]
    fn test_protect_memory_zero_length() {
        let mut data = vec![0u8; 32];
        let result = protect_memory(data.as_mut_ptr(), 0);
        assert!(result.is_err(), "CryptProtectMemory should fail with zero length");
    }

    #[test]
    fn test_page_size() {
        let page = page_size();
        assert!(page > 0, "Page size should be positive");
        assert!(page.is_power_of_two(), "Page size should be power of two");
        // Windows typically uses 4KB pages
        assert_eq!(page, 4096, "Unexpected page size");
    }

    #[test]
    fn test_allocation_granularity() {
        let gran = allocation_granularity();
        assert!(gran > 0, "Allocation granularity should be positive");
        assert!(gran.is_power_of_two(), "Allocation granularity should be power of two");
        // Windows typically uses 64KB granularity
        assert_eq!(gran, 65536, "Unexpected allocation granularity");
    }

    #[test]
    fn test_protect_and_unpreserve_content() {
        // Test that we can encrypt and decrypt content
        let original: Vec<u8> = (0..32).map(|i| i as u8).collect();
        let mut data = original.clone();

        // Protect (encrypt) the data
        protect_memory(data.as_mut_ptr(), data.len()).unwrap();

        // Data should be encrypted (different from original)
        assert_ne!(data, original, "Data should be encrypted");

        // Unprotect (decrypt) the data
        unprotect_memory(data.as_mut_ptr(), data.len()).unwrap();

        // Data should match original after decryption
        assert_eq!(data, original, "Data should be restored after decryption");
    }
}
