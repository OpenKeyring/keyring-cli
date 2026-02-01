//! File-based locking for MCP single instance
//!
//! This module provides cross-platform file locking to ensure only one MCP
//! server instance runs at a time. It uses the fs2 crate for platform-agnostic
//! file locking.

use crate::error::{Error, Result};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Lock file name
const LOCK_FILE_NAME: &str = "open-keyring-mcp.lock";

/// Get the lock file path for the current platform
///
/// # Returns
///
/// Path to the lock file:
/// - Linux/macOS: `/tmp/open-keyring-mcp.lock`
/// - Windows: `C:\Temp\open-keyring-mcp.lock`
#[cfg(unix)]
pub fn lock_file_path() -> PathBuf {
    PathBuf::from("/tmp").join(LOCK_FILE_NAME)
}

#[cfg(windows)]
pub fn lock_file_path() -> PathBuf {
    PathBuf::from("C:\\Temp").join(LOCK_FILE_NAME)
}

/// MCP file lock instance
///
/// Ensures only one MCP server instance runs at a time. The lock is
/// automatically released when the instance is dropped.
///
/// # Example
///
/// ```no_run
/// use keyring_cli::mcp::lock::McpLock;
///
/// # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
/// // Acquire lock (will fail if another instance is running)
/// let lock = McpLock::acquire()?;
///
/// // ... do work ...
///
/// // Explicitly release (optional, happens automatically on drop)
/// lock.release()?;
/// # Ok(())
/// # }
/// ```
pub struct McpLock {
    file: Option<File>,
    path: PathBuf,
}

impl McpLock {
    /// Acquire the MCP lock, waiting if necessary
    ///
    /// This will create the lock file and acquire an exclusive lock.
    /// If another instance holds the lock, this will block until
    /// the lock is released.
    ///
    /// # Returns
    ///
    /// A `McpLock` instance that holds the lock
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The lock file cannot be created or opened
    /// - The lock cannot be acquired
    /// - The PID cannot be written
    pub fn acquire() -> Result<Self> {
        let path = lock_file_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }

        // Open or create the lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(Error::Io)?;

        // Acquire exclusive lock (blocking)
        file.lock()
            .map_err(|e| Error::Mcp {
                context: format!("Failed to acquire lock: {}", e),
            })?;

        // Write our PID to the lock file
        let pid = std::process::id();
        writeln!(&file, "{}", pid).map_err(Error::Io)?;

        // Sync to ensure PID is written to disk
        file.sync_all().map_err(Error::Io)?;

        Ok(Self {
            file: Some(file),
            path,
        })
    }

    /// Try to acquire the MCP lock without blocking
    ///
    /// This will attempt to acquire the lock but return immediately
    /// with an error if another instance holds the lock.
    ///
    /// # Returns
    ///
    /// A `McpLock` instance if the lock was acquired
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The lock file cannot be created or opened
    /// - The lock is held by another instance
    /// - The PID cannot be written
    pub fn try_acquire() -> Result<Self> {
        let path = lock_file_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }

        // Open or create the lock file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(Error::Io)?;

        // Try to acquire exclusive lock (non-blocking)
        file.try_lock()
            .map_err(|e| Error::Mcp {
                context: format!("Failed to acquire lock: {}", e),
            })?;

        // Write our PID to the lock file
        let pid = std::process::id();
        writeln!(&file, "{}", pid).map_err(Error::Io)?;

        // Sync to ensure PID is written to disk
        file.sync_all().map_err(Error::Io)?;

        Ok(Self {
            file: Some(file),
            path,
        })
    }

    /// Release the lock
    ///
    /// This releases the file lock. The lock file is not deleted
    /// to avoid race conditions. The lock will be automatically
    /// released when the `McpLock` instance is dropped.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the lock was released successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be released
    pub fn release(mut self) -> Result<()> {
        if let Some(file) = self.file.take() {
            file.unlock()
                .map_err(|e| Error::Mcp {
                    context: format!("Failed to release lock: {}", e),
                })?;
        }
        Ok(())
    }

    /// Check if this instance currently holds the lock
    ///
    /// # Returns
    ///
    /// `true` if the lock is held, `false` otherwise
    pub fn is_locked(&self) -> bool {
        self.file.is_some()
    }

    /// Get the PID written to the lock file
    ///
    /// # Returns
    ///
    /// The PID of the process holding the lock, or 0 if not locked
    pub fn pid(&self) -> u32 {
        if !self.is_locked() {
            return 0;
        }

        // Try to read the PID from the lock file
        match fs::read_to_string(&self.path) {
            Ok(content) => content
                .trim()
                .parse::<u32>()
                .unwrap_or_else(|_| 0),
            Err(_) => 0,
        }
    }

    /// Get the path to the lock file
    ///
    /// # Returns
    ///
    /// The path to the lock file
    pub fn lock_file_path(&self) -> &Path {
        &self.path
    }

    /// Check if any MCP instance is currently locked
    ///
    /// This is a utility method to check lock status without acquiring.
    ///
    /// # Returns
    ///
    /// `true` if a lock is currently held by another instance
    pub fn is_locked_globally() -> bool {
        let path = lock_file_path();

        // Try to open and lock the file
        let file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(&path)
        {
            Ok(f) => f,
            Err(_) => return false, // File doesn't exist, no lock
        };

        // Try to acquire the lock
        let can_lock = file.try_lock().is_ok();

        if can_lock {
            // We acquired it, so it wasn't locked - release it
            let _ = file.unlock();
            false
        } else {
            // Couldn't acquire, so it's locked
            true
        }
    }
}

/// Check if any MCP instance is currently locked
///
/// This is a convenience method for checking global lock status.
///
/// # Returns
///
/// `true` if a lock is currently held by another instance
pub fn is_locked() -> bool {
    McpLock::is_locked_globally()
}

impl Drop for McpLock {
    fn drop(&mut self) {
        if let Some(file) = self.file.take() {
            // Best effort to unlock, ignore errors during drop
            let _ = file.unlock();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_path_unix() {
        #[cfg(unix)]
        {
            let path = lock_file_path();
            assert_eq!(path, PathBuf::from("/tmp/open-keyring-mcp.lock"));
        }
    }

    #[test]
    fn test_lock_file_path_windows() {
        #[cfg(windows)]
        {
            let path = lock_file_path();
            assert_eq!(
                path,
                PathBuf::from("C:\\Temp\\open-keyring-mcp.lock")
            );
        }
    }
}
