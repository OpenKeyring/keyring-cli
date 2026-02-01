use crate::types::SensitiveString;
use serde::{Deserialize, Serialize};

/// Record type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Password,
    SshKey,
    ApiCredential,
    Mnemonic,
    PrivateKey,
}

impl RecordType {
    /// Convert RecordType to database string representation (snake_case)
    pub fn to_db_string(self) -> &'static str {
        match self {
            RecordType::Password => "password",
            RecordType::SshKey => "ssh_key",
            RecordType::ApiCredential => "api_credential",
            RecordType::Mnemonic => "mnemonic",
            RecordType::PrivateKey => "private_key",
        }
    }

    pub fn from(s: String) -> Self {
        match s.as_str() {
            "password" => RecordType::Password,
            "ssh_key" => RecordType::SshKey,
            "api_credential" => RecordType::ApiCredential,
            "mnemonic" => RecordType::Mnemonic,
            "private_key" => RecordType::PrivateKey,
            _ => RecordType::Password, // Default
        }
    }
}

/// Stored record model (encrypted payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRecord {
    pub id: uuid::Uuid,
    pub record_type: RecordType,
    pub encrypted_data: Vec<u8>,
    pub nonce: [u8; 12],
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version number for conflict detection (incremented on each update)
    pub version: u64,
}

/// Decrypted record model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedRecord {
    pub id: uuid::Uuid,
    pub record_type: RecordType,
    pub name: String,
    pub username: Option<String>,
    pub password: SensitiveString<String>, // Wrapped in SensitiveString for auto-zeroization
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Tag model
#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Pending = 0,
    Synced = 1,
    Conflict = 2,
}

/// Sync state for a record
#[derive(Debug, Clone)]
pub struct SyncState {
    pub record_id: String,
    pub cloud_updated_at: Option<i64>,
    pub sync_status: SyncStatus,
}

/// Sync statistics aggregation
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub total: i64,
    pub pending: i64,
    pub synced: i64,
    pub conflicts: i64,
}
