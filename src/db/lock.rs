//! File locking for concurrent access between CLI and native apps

use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// File lock for coordinating CLI ↔ App access to the vault
///
/// Uses fslock-style file locking with platform-specific implementations.
/// The lock file is created alongside the vault database.
pub struct VaultLock {
    lock_file: File,
    lock_path: std::path::PathBuf,
    _held: AtomicBool,
}

impl VaultLock {
    /// Acquire an exclusive write lock
    ///
    /// # Arguments
    /// * `vault_path` - Path to the vault directory
    /// * `timeout_ms` - Maximum time to wait for lock acquisition
    ///
    /// # Returns
    /// * `Ok(VaultLock)` if lock was acquired
    /// * `Err(...)` if timeout or other error occurs
    ///
    /// # Platform behavior
    /// - **Unix/macOS**: Uses `flock` with exclusive lock
    /// - **Windows**: Uses Windows file locking
    pub fn acquire_write(vault_path: &Path, timeout_ms: u64) -> Result<Self> {
        let lock_path = vault_path.join(".lock");
        let lock_file = Self::open_lock_file(&lock_path)?;

        // Try to acquire lock with timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            match Self::try_flock_exclusive(&lock_file) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        return Err(anyhow::anyhow!(
                            "Failed to acquire write lock after {}ms: lock held by another process",
                            timeout_ms
                        ));
                    }
                    // Wait a bit before retrying
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                Err(e) => return Err(e).context("Failed to acquire write lock")?,
            }
        }

        Ok(Self {
            lock_file,
            lock_path,
            _held: AtomicBool::new(true),
        })
    }

    /// Acquire a shared read lock
    ///
    /// Multiple read locks can be held simultaneously, but write locks are exclusive.
    ///
    /// # Arguments
    /// * `vault_path` - Path to the vault directory
    /// * `timeout_ms` - Maximum time to wait for lock acquisition
    pub fn acquire_read(vault_path: &Path, timeout_ms: u64) -> Result<Self> {
        let lock_path = vault_path.join(".lock");
        let lock_file = Self::open_lock_file(&lock_path)?;

        // Try to acquire shared lock with timeout
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            match Self::try_flock_shared(&lock_file) {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        return Err(anyhow::anyhow!(
                            "Failed to acquire read lock after {}ms: write lock held by another process",
                            timeout_ms
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                Err(e) => return Err(e).context("Failed to acquire read lock")?,
            }
        }

        Ok(Self {
            lock_file,
            lock_path,
            _held: AtomicBool::new(true),
        })
    }

    /// Release the lock
    ///
    /// The lock is also automatically released when VaultLock is dropped.
    pub fn release(self) -> Result<()> {
        // Lock is released automatically when lock_file is dropped
        // and the file descriptor is closed
        Ok(())
    }

    /// Open (or create) the lock file
    fn open_lock_file(lock_path: &Path) -> Result<File> {
        use std::fs::OpenOptions;

        OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path)
            .context("Failed to open lock file")
    }

    /// Try to acquire exclusive lock (Unix/macOS)
    #[cfg(unix)]
    fn try_flock_exclusive(file: &File) -> std::io::Result<()> {
        use std::os::unix::io::AsRawFd;

        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

        if ret == 0 {
            Ok(())
        } else {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                Err(err)
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }
    }

    /// Try to acquire shared lock (Unix/macOS)
    #[cfg(unix)]
    fn try_flock_shared(file: &File) -> std::io::Result<()> {
        use std::os::unix::io::AsRawFd;

        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB) };

        if ret == 0 {
            Ok(())
        } else {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                Err(err)
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, err))
            }
        }
    }

    /// Try to acquire exclusive lock (Windows)
    #[cfg(windows)]
    fn try_flock_exclusive(file: &File) -> std::io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Storage::FileSystem::LockFileEx;
        use windows::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
        use windows::Win32::Storage::FileSystem::LOCKFILE_FAIL_IMMEDIATELY;

        let handle = file.as_raw_handle();
        unsafe {
            let mut overlapped = std::mem::zeroed();
            LockFileEx(
                handle,
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                !0,
                !0,
                &mut overlapped,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }

    /// Try to acquire shared lock (Windows)
    #[cfg(windows)]
    fn try_flock_shared(file: &File) -> std::io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Storage::FileSystem::LockFileEx;
        use windows::Win32::Storage::FileSystem::LOCKFILE_FAIL_IMMEDIATELY;

        let handle = file.as_raw_handle();
        unsafe {
            let mut overlapped = std::mem::zeroed();
            LockFileEx(
                handle,
                LOCKFILE_FAIL_IMMEDIATELY,
                0,
                !0,
                !0,
                &mut overlapped,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}

/// Implement Drop for automatic lock release
impl Drop for VaultLock {
    fn drop(&mut self) {
        if self._held.swap(false, Ordering::SeqCst) {
            // Lock is automatically released when file is closed
            // No explicit unlock needed for flock
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_path_construction() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Just test that we can construct the path
        let lock_path = vault_path.join(".lock");
        assert!(lock_path.ends_with(".lock"));
    }
}
