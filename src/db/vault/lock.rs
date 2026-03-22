//! Lock operations for vault
//!
//! This module provides functions for acquiring read/write locks on the vault.

use super::Vault;
use crate::db::lock::VaultLock;
use anyhow::Result;
use std::path::Path;

/// Open vault with an exclusive write lock
///
/// This ensures no other process (CLI or App) can write simultaneously.
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::Vault;
/// use std::path::Path;
///
/// let vault_path = Path::new("/path/to/vault");
/// let (vault, _lock) = Vault::with_write_lock(&vault_path, "password", 5000)?;
/// // ... perform write operations ...
/// // Lock is released when _lock goes out of scope
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn with_write_lock(
    path: &Path,
    master_password: &str,
    timeout_ms: u64,
) -> Result<(Vault, VaultLock)> {
    let lock = VaultLock::acquire_write(path, timeout_ms)?;
    let vault = Vault::open(path, master_password)?;
    Ok((vault, lock))
}

/// Open vault with a shared read lock
///
/// Multiple readers can hold locks simultaneously.
///
/// # Example
/// ```no_run
/// use keyring_cli::db::vault::Vault;
/// use std::path::Path;
///
/// let vault_path = Path::new("/path/to/vault");
/// let (vault, _lock) = Vault::with_read_lock(&vault_path, "password", 5000)?;
/// // ... perform read operations ...
/// // Lock is released when _lock goes out of scope
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn with_read_lock(
    path: &Path,
    master_password: &str,
    timeout_ms: u64,
) -> Result<(Vault, VaultLock)> {
    let lock = VaultLock::acquire_read(path, timeout_ms)?;
    let vault = Vault::open(path, master_password)?;
    Ok((vault, lock))
}
