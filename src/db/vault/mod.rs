// src/db/vault/mod.rs
//! Vault operations for record management

mod crypto;
mod lock;
mod metadata;
mod record;
mod search;
mod sync;

// Re-export all public functions
pub use crypto::{decrypt_password, get_record_decrypted};
pub use lock::{with_read_lock, with_write_lock};
pub use metadata::{delete_metadata, get_metadata, list_metadata_keys, set_metadata};
pub use record::{add_record, delete_record, get_record, list_records, update_record};
pub use search::{find_record_by_name, search_records};
pub use sync::{get_pending_records, get_sync_state, get_sync_stats, mark_record_pending, set_sync_state};

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

use super::models::{DecryptedRecord, StoredRecord, SyncState};
use super::SyncStats;
use crate::types::SensitiveString;

/// Vault for managing encrypted password records
pub struct Vault {
    /// Database connection (public for testing purposes)
    pub conn: Connection,
}

impl Vault {
    /// Open or create a vault at the specified path
    ///
    /// For write operations, callers should use `Vault::with_write_lock()`
    /// to ensure proper locking. Read-only operations can use `Vault::open()`.
    pub fn open(path: &Path, _master_password: &str) -> Result<Self> {
        let conn = super::schema::initialize_database(path)?;
        Ok(Self { conn })
    }

    // ==================== Record Operations ====================

    /// List all non-deleted records with tags
    pub fn list_records(&self) -> Result<Vec<StoredRecord>> {
        record::list_records(&self.conn)
    }

    /// Get a specific record by ID with tags
    pub fn get_record(&self, id: &str) -> Result<StoredRecord> {
        record::get_record(&self.conn, id)
    }

    /// Add a new record with tag support
    pub fn add_record(&mut self, record: &StoredRecord) -> Result<()> {
        record::add_record(&mut self.conn, record)
    }

    /// Update an existing record with version increment
    pub fn update_record(&mut self, record: &StoredRecord) -> Result<()> {
        record::update_record(&mut self.conn, record)
    }

    /// Delete a record (soft delete)
    pub fn delete_record(&mut self, id: &str) -> Result<()> {
        record::delete_record(&mut self.conn, id)
    }

    // ==================== Metadata Operations ====================

    /// Set a metadata key-value pair
    pub fn set_metadata(&mut self, key: &str, value: &str) -> Result<()> {
        metadata::set_metadata(&mut self.conn, key, value)
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        metadata::get_metadata(&self.conn, key)
    }

    /// Delete metadata value by key
    pub fn delete_metadata(&mut self, key: &str) -> Result<()> {
        metadata::delete_metadata(&mut self.conn, key)
    }

    /// List all metadata keys matching a prefix
    pub fn list_metadata_keys(&self, prefix: &str) -> Result<Vec<String>> {
        metadata::list_metadata_keys(&self.conn, prefix)
    }

    // ==================== Sync Operations ====================

    /// Get sync state for a record
    pub fn get_sync_state(&self, record_id: &str) -> Result<Option<SyncState>> {
        sync::get_sync_state(&self.conn, record_id)
    }

    /// Set sync state for a record
    pub fn set_sync_state(
        &mut self,
        record_id: &str,
        cloud_updated_at: Option<i64>,
        sync_status: super::models::SyncStatus,
    ) -> Result<()> {
        sync::set_sync_state(&mut self.conn, record_id, cloud_updated_at, sync_status)
    }

    /// Mark record as pending sync
    pub fn mark_record_pending(&mut self, record_id: &str) -> Result<()> {
        sync::mark_record_pending(&mut self.conn, record_id)
    }

    /// Get sync statistics for all records
    pub fn get_sync_stats(&self) -> Result<SyncStats> {
        sync::get_sync_stats(&self.conn)
    }

    /// Get all records with pending sync status
    pub fn get_pending_records(&self) -> Result<Vec<StoredRecord>> {
        sync::get_pending_records(&self.conn)
    }

    // ==================== Search Operations ====================

    /// Search records by pattern matching
    pub fn search_records(&self, query: &str) -> Result<Vec<StoredRecord>> {
        search::search_records(&self.conn, query)
    }

    /// Find a record by its decrypted name
    pub fn find_record_by_name(&self, name: &str) -> Result<Option<StoredRecord>> {
        search::find_record_by_name(&self.conn, name)
    }

    // ==================== Crypto Operations ====================

    /// Decrypt the password field from a stored record
    pub fn decrypt_password(
        &self,
        record: &StoredRecord,
        dek: &[u8],
    ) -> Result<SensitiveString<String>> {
        crypto::decrypt_password(&self.conn, record, dek)
    }

    /// Get a decrypted record by ID
    pub fn get_record_decrypted(&self, id: &str, dek: &[u8]) -> Result<DecryptedRecord> {
        crypto::get_record_decrypted(&self.conn, id, dek)
    }

    // ==================== Lock Operations ====================

    /// Open vault with an exclusive write lock
    pub fn with_write_lock(
        path: &Path,
        master_password: &str,
        timeout_ms: u64,
    ) -> Result<(Self, super::lock::VaultLock)> {
        lock::with_write_lock(path, master_password, timeout_ms)
    }

    /// Open vault with a shared read lock
    pub fn with_read_lock(
        path: &Path,
        master_password: &str,
        timeout_ms: u64,
    ) -> Result<(Self, super::lock::VaultLock)> {
        lock::with_read_lock(path, master_password, timeout_ms)
    }
}

// Re-export decode_nonce for internal use
pub(crate) fn decode_nonce(bytes: &[u8]) -> Result<[u8; 12]> {
    if bytes.len() != 12 {
        return Err(anyhow::anyhow!("Invalid nonce length: {}", bytes.len()));
    }
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(bytes);
    Ok(nonce)
}

#[cfg(test)]
mod tests;
