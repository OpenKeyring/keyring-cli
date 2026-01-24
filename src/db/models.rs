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

/// Database record model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub record_type: RecordType,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: i64,
    pub updated_at: i64,
    pub updated_by: String,
    pub version: u64,
    pub deleted: bool,
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
