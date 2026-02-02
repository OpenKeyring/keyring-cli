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

#[cfg(test)]
mod tests {
    use super::*;

    // RecordType tests
    #[test]
    fn test_record_type_to_db_string() {
        assert_eq!(RecordType::Password.to_db_string(), "password");
        assert_eq!(RecordType::SshKey.to_db_string(), "ssh_key");
        assert_eq!(RecordType::ApiCredential.to_db_string(), "api_credential");
        assert_eq!(RecordType::Mnemonic.to_db_string(), "mnemonic");
        assert_eq!(RecordType::PrivateKey.to_db_string(), "private_key");
    }

    #[test]
    fn test_record_type_from_all_valid_values() {
        assert_eq!(RecordType::from("password".to_string()), RecordType::Password);
        assert_eq!(RecordType::from("ssh_key".to_string()), RecordType::SshKey);
        assert_eq!(RecordType::from("api_credential".to_string()), RecordType::ApiCredential);
        assert_eq!(RecordType::from("mnemonic".to_string()), RecordType::Mnemonic);
        assert_eq!(RecordType::from("private_key".to_string()), RecordType::PrivateKey);
    }

    #[test]
    fn test_record_type_from_invalid_value_defaults_to_password() {
        assert_eq!(RecordType::from("invalid_type".to_string()), RecordType::Password);
        assert_eq!(RecordType::from("".to_string()), RecordType::Password);
        assert_eq!(RecordType::from("random-value".to_string()), RecordType::Password);
    }

    #[test]
    fn test_record_type_roundtrip() {
        let original = RecordType::ApiCredential;
        let db_string = original.to_db_string();
        let restored = RecordType::from(db_string.to_string());
        assert_eq!(original, restored);
    }

    // SyncStatus tests
    #[test]
    fn test_sync_status_numeric_values() {
        assert_eq!(SyncStatus::Pending as i32, 0);
        assert_eq!(SyncStatus::Synced as i32, 1);
        assert_eq!(SyncStatus::Conflict as i32, 2);
    }

    #[test]
    fn test_sync_status_equality() {
        assert_eq!(SyncStatus::Pending, SyncStatus::Pending);
        assert_ne!(SyncStatus::Pending, SyncStatus::Synced);
        assert_ne!(SyncStatus::Synced, SyncStatus::Conflict);
    }

    // StoredRecord creation tests
    #[test]
    fn test_stored_record_creation() {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let record = StoredRecord {
            id,
            record_type: RecordType::Password,
            encrypted_data: vec![1, 2, 3, 4],
            nonce: [0u8; 12],
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_at: now,
            updated_at: now,
            version: 1,
        };

        assert_eq!(record.id, id);
        assert_eq!(record.version, 1);
        assert_eq!(record.tags.len(), 2);
    }

    #[test]
    fn test_stored_record_serialization() {
        let record = StoredRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Password,
            encrypted_data: vec![1, 2, 3],
            nonce: [5u8; 12],
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 5,
        };

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&record).unwrap();
        let deserialized: StoredRecord = serde_json::from_str(&serialized).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.record_type, deserialized.record_type);
        assert_eq!(record.encrypted_data, deserialized.encrypted_data);
        assert_eq!(record.nonce, deserialized.nonce);
        assert_eq!(record.version, deserialized.version);
    }

    // DecryptedRecord tests
    #[test]
    fn test_decrypted_record_with_sensitive_string() {
        let password = SensitiveString::new("secret-password".to_string());
        let record = DecryptedRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Password,
            name: "test-record".to_string(),
            username: Some("user@example.com".to_string()),
            password,
            url: Some("https://example.com".to_string()),
            notes: Some("important notes".to_string()),
            tags: vec!["work".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert_eq!(record.password.get(), "secret-password");
        assert!(record.username.is_some());
        assert!(record.url.is_some());
    }

    #[test]
    fn test_decrypted_record_with_none_optional_fields() {
        let record = DecryptedRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::PrivateKey,
            name: "minimal-record".to_string(),
            username: None,
            password: SensitiveString::new("key-data".to_string()),
            url: None,
            notes: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(record.username.is_none());
        assert!(record.url.is_none());
        assert!(record.notes.is_none());
        assert!(record.tags.is_empty());
    }

    // Tag tests
    #[test]
    fn test_tag_creation() {
        let tag = Tag {
            id: 123,
            name: "important".to_string(),
        };

        assert_eq!(tag.id, 123);
        assert_eq!(tag.name, "important");
    }

    // SyncState tests
    #[test]
    fn test_sync_state_creation() {
        let state = SyncState {
            record_id: "record-123".to_string(),
            cloud_updated_at: Some(1234567890),
            sync_status: SyncStatus::Synced,
        };

        assert_eq!(state.record_id, "record-123");
        assert_eq!(state.cloud_updated_at, Some(1234567890));
        assert_eq!(state.sync_status, SyncStatus::Synced);
    }

    #[test]
    fn test_sync_state_with_pending_status() {
        let state = SyncState {
            record_id: "record-456".to_string(),
            cloud_updated_at: None,
            sync_status: SyncStatus::Pending,
        };

        assert_eq!(state.sync_status, SyncStatus::Pending);
        assert!(state.cloud_updated_at.is_none());
    }

    // SyncStats tests
    #[test]
    fn test_sync_stats_creation() {
        let stats = SyncStats {
            total: 100,
            pending: 20,
            synced: 75,
            conflicts: 5,
        };

        assert_eq!(stats.total, 100);
        assert_eq!(stats.pending, 20);
        assert_eq!(stats.synced, 75);
        assert_eq!(stats.conflicts, 5);
    }

    #[test]
    fn test_sync_stats_sum_consistency() {
        let stats = SyncStats {
            total: 50,
            pending: 10,
            synced: 35,
            conflicts: 5,
        };

        // Verify pending + synced + conflicts <= total
        assert!(stats.pending + stats.synced + stats.conflicts <= stats.total);
    }
}
