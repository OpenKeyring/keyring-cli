//! Crypto operations for vault records
//!
//! This module provides functions for decrypting vault records.

use crate::db::models::DecryptedRecord;
use crate::db::models::StoredRecord;
use crate::types::SensitiveString;
use anyhow::Result;
use rusqlite::Connection;

/// Decrypt the password field from a stored record
///
/// This function decrypts the encrypted_data field of a record using the provided DEK
/// and returns the password wrapped in a SensitiveString for automatic zeroization.
///
/// # Arguments
/// * `_conn` - Database connection (unused but kept for API consistency)
/// * `record` - The stored record containing encrypted data
/// * `dek` - The Data Encryption Key (32 bytes)
///
/// # Returns
/// The decrypted password wrapped in SensitiveString
///
/// # Security Note
/// The returned SensitiveString will automatically zeroize its contents when dropped,
/// preventing sensitive password data from remaining in memory.
pub fn decrypt_password(
    _conn: &Connection,
    record: &StoredRecord,
    dek: &[u8],
) -> Result<SensitiveString<String>> {
    // Convert DEK slice to array
    let dek_array: [u8; 32] = dek
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid DEK length: expected 32 bytes"))?;

    // Decrypt using the crypto module (ciphertext, nonce, key)
    let decrypted =
        crate::crypto::aes256gcm::decrypt(&record.encrypted_data, &record.nonce, &dek_array)?;

    // Parse the decrypted JSON to extract the password field
    let json_str = String::from_utf8(decrypted)?;
    let payload: serde_json::Value = serde_json::from_str(&json_str)?;

    // Extract the password field
    let password = payload
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No password field in decrypted payload"))?;

    Ok(SensitiveString::new(password.to_string()))
}

/// Get a decrypted record by ID
///
/// This function retrieves a stored record, decrypts it using the provided DEK,
/// and returns a DecryptedRecord with the password field wrapped in SensitiveString.
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - The UUID of the record to decrypt
/// * `dek` - The Data Encryption Key (32 bytes)
///
/// # Returns
/// A DecryptedRecord with decrypted data, password wrapped in SensitiveString
pub fn get_record_decrypted(conn: &Connection, id: &str, dek: &[u8]) -> Result<DecryptedRecord> {
    // Get the stored record
    let stored = super::record::get_record(conn, id)?;

    // Convert DEK slice to array
    let dek_array: [u8; 32] = dek
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid DEK length: expected 32 bytes"))?;

    // Decrypt the record data
    let decrypted =
        crate::crypto::aes256gcm::decrypt(&stored.encrypted_data, &stored.nonce, &dek_array)?;
    let json_str = String::from_utf8(decrypted)?;

    // Parse the record payload
    #[derive(serde::Deserialize)]
    struct RecordPayload {
        name: String,
        username: Option<String>,
        password: String,
        url: Option<String>,
        notes: Option<String>,
    }

    let payload: RecordPayload = serde_json::from_str(&json_str)?;

    Ok(DecryptedRecord {
        id: stored.id,
        name: payload.name,
        record_type: stored.record_type,
        username: payload.username,
        password: SensitiveString::new(payload.password),
        url: payload.url,
        notes: payload.notes,
        tags: stored.tags,
        created_at: stored.created_at,
        updated_at: stored.updated_at,
    })
}
