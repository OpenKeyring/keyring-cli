//! Tests for MCP file locking mechanism
//!
//! This module tests the file-based locking that ensures only one MCP instance
//! runs at a time.
//!
//! IMPORTANT: These tests must be run with --test-threads=1 to avoid interference
//! since they all manipulate the same global lock file.
//!
//! Run with: cargo test --test mcp_lock_test -- --test-threads=1

use keyring_cli::mcp::lock::{is_locked, McpLock};
use serial_test::serial;
use std::thread;

#[serial]
#[test]
fn test_lock_acquisition() {
    // First lock should succeed
    let lock1 = McpLock::acquire().expect("First lock should succeed");
    assert!(lock1.is_locked(), "Lock should be held");

    // Second lock attempt should fail
    let lock2_result = McpLock::try_acquire();
    assert!(lock2_result.is_err(), "Second lock should fail");

    // Release first lock
    lock1.release().expect("Release should succeed");

    // Now second lock should succeed
    let lock2 = McpLock::acquire().expect("Second lock should succeed after first release");
    lock2.release().expect("Second release should succeed");
}

#[serial]
#[test]
fn test_try_acquire() {
    // No lock held initially
    let lock1 = McpLock::try_acquire().expect("First try_acquire should succeed");
    assert!(lock1.is_locked(), "Lock should be held");

    // Second attempt should fail
    let lock2_result = McpLock::try_acquire();
    assert!(lock2_result.is_err(), "Second try_acquire should fail");

    lock1.release().expect("Release should succeed");
}

#[serial]
#[test]
fn test_pid_writing() {
    let lock = McpLock::acquire().expect("Lock should be acquired");
    let pid = lock.pid();
    assert!(pid > 0, "PID should be positive");

    // Current PID should match
    let current_pid = std::process::id();
    assert_eq!(pid, current_pid, "Lock PID should match current process");

    lock.release().expect("Release should succeed");
}

#[serial]
#[test]
fn test_double_release() {
    let lock = McpLock::acquire().expect("Lock should be acquired");

    // First release should succeed (takes ownership)
    lock.release().expect("First release should succeed");

    // After release, lock is gone - can't call release again
    // The Drop trait has already been called during release
}

#[serial]
#[test]
fn test_drop_auto_release() {
    // Test that Drop trait automatically releases the lock
    {
        let lock = McpLock::acquire().expect("Lock should be acquired");
        assert!(lock.is_locked(), "Lock should be held");
        // Lock goes out of scope and Drop is called
    }

    // After drop, we should be able to acquire again
    let lock2 = McpLock::try_acquire().expect("Lock should be available after drop");
    lock2.release().expect("Release should succeed");
}

#[serial]
#[test]
fn test_concurrent_lock_attempts() {
    let lock1 = McpLock::acquire().expect("First lock should succeed");

    // Try to acquire in a separate thread
    let handle = thread::spawn(|| {
        // This should fail since lock1 is held
        let lock_result = McpLock::try_acquire();
        assert!(
            lock_result.is_err(),
            "Lock acquisition in thread should fail"
        );
    });

    handle.join().expect("Thread should complete");

    lock1.release().expect("Release should succeed");
}

#[serial]
#[test]
fn test_lock_file_path() {
    let lock = McpLock::acquire().expect("Lock should be acquired");
    let path = lock.lock_file_path();

    // Path should contain the lock file name
    assert!(
        path.to_string_lossy().contains("open-keyring-mcp.lock"),
        "Lock file path should contain 'open-keyring-mcp.lock'"
    );

    lock.release().expect("Release should succeed");
}

#[serial]
#[test]
fn test_is_locked() {
    // Initially no lock
    assert!(!is_locked(), "No lock should be held initially");

    // After acquiring
    let lock = McpLock::acquire().expect("Lock should be acquired");
    assert!(is_locked(), "Lock should be held");

    // After releasing
    lock.release().expect("Release should succeed");
    assert!(!is_locked(), "No lock should be held after release");
}
