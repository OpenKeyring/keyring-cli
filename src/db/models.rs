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

/// Database record model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: uuid::Uuid,
    pub record_type: RecordType,
    pub encrypted_data: String,
    pub name: String,
    pub username: Option<String>,
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
