//! Linux-specific platform functionality
//!
//! Implements memory protection using mlock system call.

use crate::error::Result;
use crate::platform::PlatformError;

/// Protect memory from being swapped to disk using mlock
///
/// This function prevents sensitive data (like passwords, encryption keys)
/// from being written to disk by locking the memory pages in RAM.
///
/// # Arguments
/// * `addr` - Pointer to the memory region to protect
/// * `len` - Length of the memory region in bytes
///
/// # Returns
/// * `Ok(())` if memory was successfully protected
/// * `Err(PlatformError)` if mlock failed
///
/// # Safety
/// The caller must ensure that the memory region is valid and accessible.
pub fn protect_memory(addr: *mut u8, len: usize) -> Result<()> {
    if addr.is_null() || len == 0 {
        return Err(
            PlatformError::MemoryProtectionFailed("Invalid address or length".to_string()).into(),
        );
    }

    // Call mlock to lock memory pages
    let result = unsafe { libc::mlock(addr as *const libc::c_void, len) };

    if result != 0 {
        let errno = unsafe { *libc::__errno_location() };
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "mlock failed with errno {}: {}",
            errno,
            std::io::Error::from_raw_os_error(errno)
        ))
        .into());
    }

    Ok(())
}

/// Unlock previously locked memory using munlock
///
/// This should be called when the protected memory is no longer needed.
/// Note: This is optional; memory will be automatically unlocked when freed.
///
/// # Arguments
/// * `addr` - Pointer to the memory region to unlock
/// * `len` - Length of the memory region in bytes
///
/// # Returns
/// * `Ok(())` if memory was successfully unlocked
/// * `Err(PlatformError)` if munlock failed
///
/// # Safety
/// The caller must ensure that the memory region was previously locked.
pub fn unprotect_memory(addr: *mut u8, len: usize) -> Result<()> {
    if addr.is_null() || len == 0 {
        return Err(
            PlatformError::MemoryProtectionFailed("Invalid address or length".to_string()).into(),
        );
    }

    let result = unsafe { libc::munlock(addr as *const libc::c_void, len) };

    if result != 0 {
        let errno = unsafe { *libc::__errno_location() };
        return Err(PlatformError::MemoryProtectionFailed(format!(
            "munlock failed with errno {}: {}",
            errno,
            std::io::Error::from_raw_os_error(errno)
        ))
        .into());
    }

    Ok(())
}

/// Get the system page size for memory alignment
///
/// Memory protection operations work on page boundaries.
/// Returns the system page size in bytes.
pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_memory_small() {
        let mut data = vec![0u8; 100];
        let result = protect_memory(data.as_mut_ptr(), data.len());
        assert!(result.is_ok(), "mlock should succeed for small allocations");

        // Cleanup
        let _ = unprotect_memory(data.as_mut_ptr(), data.len());
    }

    #[test]
    fn test_protect_memory_null_pointer() {
        let result = protect_memory(std::ptr::null_mut(), 100);
        assert!(result.is_err(), "mlock should fail with null pointer");
    }

    #[test]
    fn test_protect_memory_zero_length() {
        let mut data = vec![0u8; 100];
        let result = protect_memory(data.as_mut_ptr(), 0);
        assert!(result.is_err(), "mlock should fail with zero length");
    }

    #[test]
    fn test_unprotect_memory() {
        let mut data = vec![0u8; 100];
        protect_memory(data.as_mut_ptr(), data.len()).unwrap();
        let result = unprotect_memory(data.as_mut_ptr(), data.len());
        assert!(result.is_ok(), "munlock should succeed");
    }

    #[test]
    fn test_page_size() {
        let page = page_size();
        assert!(page > 0, "Page size should be positive");
        assert!(page.is_power_of_two(), "Page size should be power of two");
    }

    #[test]
    fn test_protect_aligned_memory() {
        // Test with page-aligned allocation
        let page = page_size();
        let mut data = vec![0u8; page * 2]; // Allocate 2 pages

        // Align to page boundary
        let addr = data.as_mut_ptr();
        let aligned_addr = if addr as usize % page != 0 {
            ((addr as usize / page + 1) * page) as *mut u8
        } else {
            addr
        };

        let result = protect_memory(aligned_addr, page);
        assert!(
            result.is_ok(),
            "mlock should succeed for page-aligned memory"
        );

        // Cleanup
        let _ = unprotect_memory(aligned_addr, page);
    }
}
