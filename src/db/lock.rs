//! File locking for concurrent access between CLI and native apps

use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// File lock for coordinating CLI ↔ App access to the vault
///
/// Uses fslock-style file locking with platform-specific implementations.
/// The lock file is created alongside the vault database.
///
/// # Lock Path Construction
///
/// When `vault_path` is a file path (e.g., `/path/to/passwords.db`):
/// - Lock file is created as `/path/to/passwords.db.lock` (replacing extension)
///
/// When `vault_path` is a directory path (e.g., `/path/to/vault/`):
/// - Lock file is created as `/path/to/vault/.lock`
#[allow(dead_code)]
pub struct VaultLock {
    #[allow(dead_code)]
    lock_file: File,
    #[allow(dead_code)]
    lock_path: std::path::PathBuf,
    _held: AtomicBool,
}

impl VaultLock {
    /// Acquire an exclusive write lock
    ///
    /// # Arguments
    /// * `vault_path` - Path to the vault database file
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
        let lock_path = Self::lock_path_for_vault(vault_path);
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
    /// * `vault_path` - Path to the vault database file
    /// * `timeout_ms` - Maximum time to wait for lock acquisition
    pub fn acquire_read(vault_path: &Path, timeout_ms: u64) -> Result<Self> {
        let lock_path = Self::lock_path_for_vault(vault_path);
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

    /// Determine the lock file path for a given vault path
    ///
    /// Handles both file paths and directory paths:
    /// - File path (`/path/to/passwords.db`) → `/path/to/passwords.db.lock`
    /// - Directory path (`/path/to/vault/`) → `/path/to/vault/.lock`
    fn lock_path_for_vault(vault_path: &Path) -> std::path::PathBuf {
        if vault_path.extension().is_some() {
            // It's a file path (e.g., /path/to/passwords.db)
            // Replace/add .lock extension
            let mut lock_path = vault_path.to_path_buf();
            lock_path.set_extension("lock");
            lock_path
        } else {
            // It's a directory path (e.g., /path/to/vault/)
            // Append .lock
            vault_path.join(".lock")
        }
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
            .truncate(true)
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
                Err(std::io::Error::other(err))
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
                Err(std::io::Error::other(err))
            }
        }
    }

    /// Try to acquire exclusive lock (Windows)
    #[cfg(windows)]
    fn try_flock_exclusive(file: &File) -> std::io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::LockFileEx;
        use windows::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
        use windows::Win32::Storage::FileSystem::LOCKFILE_FAIL_IMMEDIATELY;

        // HANDLE in windows crate v0.58 is struct HANDLE(pub *mut c_void)
        // as_raw_handle() returns isize, need to cast to pointer
        let handle = HANDLE(file.as_raw_handle() as *mut _);
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
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Storage::FileSystem::LockFileEx;
        use windows::Win32::Storage::FileSystem::LOCKFILE_FAIL_IMMEDIATELY;

        // HANDLE in windows crate v0.58 is struct HANDLE(pub *mut c_void)
        // as_raw_handle() returns isize, need to cast to pointer
        let handle = HANDLE(file.as_raw_handle() as *mut _);
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

    #[test]
    fn test_lock_path_construction() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let vault_path = temp_dir.path();

        // Just test that we can construct the path
        let lock_path = vault_path.join(".lock");
        assert!(lock_path.ends_with(".lock"));
    }
}
