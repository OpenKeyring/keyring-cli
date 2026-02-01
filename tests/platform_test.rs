//! Platform detection and memory protection tests
//!
//! This test suite verifies platform-specific functionality including:
//! - Memory protection (mlock/CryptProtectMemory)
//! - SSH binary detection
//! - Platform-specific utilities

use keyring_cli::platform::{has_ssh, page_size, protect_memory, unprotect_memory, which_ssh};

#[test]
fn test_ssh_detection() {
    // This test checks if SSH binary can be detected
    // May be skipped in CI environments without SSH
    let ssh_path = which_ssh();

    if let Some(path) = ssh_path {
        println!("Found SSH at: {}", path);
        assert!(!path.is_empty(), "SSH path should not be empty");

        // Verify the path exists
        #[cfg(unix)]
        {
            use std::path::Path;
            assert!(Path::new(&path).exists(), "SSH path should exist: {}", path);
        }

        #[cfg(target_os = "windows")]
        {
            use std::path::Path;
            assert!(Path::new(&path).exists(), "SSH path should exist: {}", path);
        }
    } else {
        println!("SSH not found on this system");
    }
}

#[test]
fn test_has_ssh_consistency() {
    // has_ssh should be consistent with which_ssh
    let ssh_path = which_ssh();
    assert_eq!(has_ssh(), ssh_path.is_some());
}

#[test]
fn test_page_size() {
    let page = page_size();
    assert!(page > 0, "Page size should be positive");
    assert!(
        page.is_power_of_two(),
        "Page size should be power of two, got: {}",
        page
    );

    // Common page sizes are 4KB, 8KB, 16KB, or 64KB
    assert!(
        [4096, 8192, 16384, 65536].contains(&page),
        "Page size {} is not a common value",
        page
    );
}

#[test]
fn test_protect_memory_small() {
    // Test protecting a small allocation (100 bytes)
    let mut data = vec![0u8; 100];

    #[cfg(unix)]
    {
        let result = protect_memory(data.as_mut_ptr(), data.len());
        assert!(
            result.is_ok(),
            "protect_memory should succeed for small allocations: {:?}",
            result
        );

        // Cleanup
        let _ = unprotect_memory(data.as_mut_ptr(), data.len());
    }

    #[cfg(target_os = "windows")]
    {
        // Windows requires length to be a multiple of 16 bytes
        let mut data = vec![0u8; 112]; // 7 * 16
        let result = protect_memory(data.as_mut_ptr(), data.len());
        assert!(
            result.is_ok(),
            "protect_memory should succeed for aligned allocations: {:?}",
            result
        );

        // Cleanup
        let _ = unprotect_memory(data.as_mut_ptr(), data.len());
    }
}

#[test]
fn test_protect_memory_null_pointer() {
    let result = protect_memory(std::ptr::null_mut(), 100);
    assert!(
        result.is_err(),
        "protect_memory should fail with null pointer"
    );
}

#[test]
fn test_protect_memory_zero_length() {
    let mut data = vec![0u8; 100];
    let result = protect_memory(data.as_mut_ptr(), 0);
    assert!(
        result.is_err(),
        "protect_memory should fail with zero length"
    );
}

#[test]
fn test_protect_unprotect_cycle() {
    // Test that we can protect and then unprotect memory

    #[cfg(unix)]
    let mut data = vec![42u8; 256];

    #[cfg(target_os = "windows")]
    let mut data = vec![42u8; 256]; // 256 = 16 * 16

    protect_memory(data.as_mut_ptr(), data.len()).expect("mlock should succeed");

    // Verify data is still accessible (on Unix)
    #[cfg(unix)]
    assert_eq!(
        data,
        vec![42u8; 256],
        "Data should be unchanged after mlock"
    );

    // On Windows, data will be encrypted, so we can't verify it directly

    unprotect_memory(data.as_mut_ptr(), data.len()).expect("munlock should succeed");

    // After unprotecting, data should be restored
    assert_eq!(
        data,
        vec![42u8; 256],
        "Data should be unchanged after unprotect"
    );
}

#[test]
fn test_multiple_protection_cycles() {
    // Test that we can protect/unprotect multiple times

    #[cfg(unix)]
    let mut data = vec![0u8; 200];

    #[cfg(target_os = "windows")]
    let mut data = vec![0u8; 208]; // 13 * 16

    for i in 0..5 {
        // Protect
        protect_memory(data.as_mut_ptr(), data.len())
            .expect(&format!("Iteration {}: protect should succeed", i));

        // Unprotect
        unprotect_memory(data.as_mut_ptr(), data.len())
            .expect(&format!("Iteration {}: unprotect should succeed", i));
    }
}

#[test]
fn test_protect_large_allocation() {
    // Test protecting a larger allocation
    // Note: macOS has strict limits on mlock, so this may fail

    #[cfg(unix)]
    let size = 16 * 1024; // 16KB

    #[cfg(target_os = "windows")]
    let size = 16 * 1024; // 16KB (multiple of 16)

    let mut data = vec![0u8; size];

    let result = protect_memory(data.as_mut_ptr(), data.len());

    // On macOS, this may fail due to resource limits
    #[cfg(target_os = "macos")]
    {
        if result.is_err() {
            println!(
                "Warning: Large allocation protection failed on macOS (expected due to limits)"
            );
            return;
        }
    }

    assert!(
        result.is_ok(),
        "protect_memory should succeed for larger allocations: {:?}",
        result
    );

    // Cleanup
    let _ = unprotect_memory(data.as_mut_ptr(), data.len());
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_length_validation() {
    use keyring_cli::platform::PlatformError;

    // Windows requires length to be a multiple of 16 bytes
    let mut data = vec![0u8; 15]; // Not a multiple of 16

    let result = protect_memory(data.as_mut_ptr(), data.len());
    assert!(
        matches!(result, Err(PlatformError::MemoryProtectionFailed(_))),
        "protect_memory should fail with invalid length on Windows"
    );
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_max_locked_memory() {
    // Test querying the maximum locked memory on macOS
    use keyring_cli::platform::max_locked_memory;

    let max = max_locked_memory();
    if max > 0 {
        println!(
            "macOS max locked memory: {} bytes ({} MB)",
            max,
            max / 1024 / 1024
        );
        assert!(max > 0, "Max locked memory should be positive");
    }
}

#[test]
#[cfg(unix)]
fn test_page_aligned_protection() {
    use keyring_cli::platform::page_size;

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
        "protect_memory should succeed for page-aligned memory: {:?}",
        result
    );

    // Cleanup
    let _ = unprotect_memory(aligned_addr, page);
}

// Integration test: Verify that memory protection actually prevents swapping
// Note: This test is difficult to verify reliably and is typically skipped
#[test]
#[ignore]
fn test_memory_prevents_swap() {
    // This test would need to:
    // 1. Allocate and protect memory
    // 2. Fill it with sensitive data
    // 3. Force memory pressure (not portable)
    // 4. Verify data is not in swap (requires root/admin)
    //
    // In practice, this is verified through external tools like:
    // - Linux: check /proc/self/status for VmLck field
    // - macOS: use vmmap or other tools
    // - Windows: use task manager or Process Explorer

    println!("Memory swap prevention test is ignored - requires external verification");
}
