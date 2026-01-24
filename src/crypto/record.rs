//! Encrypted record payload helpers

use crate::crypto::CryptoManager;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPayload {
    pub name: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

pub fn encrypt_payload(
    crypto: &CryptoManager,
    payload: &RecordPayload,
) -> Result<(Vec<u8>, [u8; 12])> {
    let bytes = serde_json::to_vec(payload)?;
    crypto.encrypt(&bytes)
}

pub fn decrypt_payload(
    crypto: &CryptoManager,
    ciphertext: &[u8],
    nonce: &[u8; 12],
) -> Result<RecordPayload> {
    let plaintext = crypto.decrypt(ciphertext, nonce)?;
    Ok(serde_json::from_slice(&plaintext)?)
}
