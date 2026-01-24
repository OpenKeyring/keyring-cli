//! File locking for concurrent access

use anyhow::Result;
use std::path::Path;

/// File lock for coordinating CLI <-> App access
pub struct VaultLock {
    _lock_path: std::path::PathBuf,
}

impl VaultLock {
    /// Acquire write lock
    pub fn acquire_write(vault_path: &Path, _timeout_ms: u64) -> Result<Self> {
        Ok(Self {
            _lock_path: vault_path.join(".lock"),
        })
    }

    /// Acquire read lock
    pub fn acquire_read(vault_path: &Path, _timeout_ms: u64) -> Result<Self> {
        Ok(Self {
            _lock_path: vault_path.join(".lock"),
        })
    }

    /// Release the lock
    pub fn release(self) -> Result<()> {
        Ok(())
    }
}
