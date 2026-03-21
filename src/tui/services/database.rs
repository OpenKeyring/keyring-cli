//! TUI 数据库服务适配器
//!
//! 封装现有 Vault 实现，提供 TUI 层所需的 DatabaseService trait。
//! 集成真实的 SQLite 数据库持久化（通过 db::Vault）。

use crate::db::{Vault, models::{DecryptedRecord, RecordType, StoredRecord}};
use crate::tui::error::{ErrorKind, TuiError, TuiResult};
use crate::tui::traits::{DatabaseService, SecureClear};
use crate::tui::models::password::PasswordRecord;
use crate::crypto::aes256gcm;
use crate::crypto::record::RecordPayload;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// TUI 数据库服务
///
/// 适配现有的 Vault 实现，为 TUI 层提供统一的数据库访问接口。
pub struct TuiDatabaseService {
    /// Vault 实例（需要 mut 用于写操作）
    vault: Arc<Mutex<Vault>>,
    /// 数据加密密钥（用于解密记录）
    dek: Option<[u8; 32]>,
}

impl std::fmt::Debug for TuiDatabaseService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TuiDatabaseService")
            .field("vault", &"Arc<Mutex<Vault>>")
            .field("dek", &self.dek.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

impl TuiDatabaseService {
    /// 创建新的数据库服务（无 Vault 连接）
    pub fn new() -> Self {
        Self {
            vault: Arc::new(Mutex::new(Vault::open(
                &std::path::PathBuf::from(":memory:"),
                ""
            ).unwrap_or_else(|_| Vault::open(&std::path::PathBuf::from(":memory:"), "").unwrap()))),
            dek: None,
        }
    }

    /// 使用 Vault 实例创建服务
    pub fn with_vault(vault: Arc<Mutex<Vault>>) -> Self {
        Self {
            vault,
            dek: None,
        }
    }

    /// 设置数据加密密钥
    pub fn with_dek(mut self, dek: [u8; 32]) -> Self {
        self.dek = Some(dek);
        self
    }

    /// 将 DecryptedRecord 转换为 PasswordRecord
    fn decrypted_to_password(record: &DecryptedRecord) -> PasswordRecord {
        let mut pw = PasswordRecord::new(
            record.id.to_string(),
            record.name.clone(),
            record.password.get().clone(),
        )
        .with_username(record.username.clone().unwrap_or_default())
        .with_url(record.url.clone().unwrap_or_default())
        .with_notes(record.notes.clone().unwrap_or_default())
        .with_tags(record.tags.clone());

        if record.deleted {
            pw.is_deleted = true;
            pw.deleted_at = Some(record.updated_at);
        }

        pw
    }

    /// 检查服务是否已初始化（有 DEK）
    fn is_initialized(&self) -> bool {
        self.dek.is_some()
    }
}

impl Default for TuiDatabaseService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseService for TuiDatabaseService {
    /// 根据 ID 获取密码记录
    async fn get_password(&self, _id: &str) -> TuiResult<()> {
        // 当前 trait 返回 ()，暂时返回 Ok
        // 实际实现需要在 trait 更新后修改
        Ok(())
    }

    /// 保存密码记录
    async fn save_password(&self, _record: &()) -> TuiResult<()> {
        // 当前 trait 参数为 ()，暂时返回 Ok
        // 实际实现需要在 trait 更新后修改
        Ok(())
    }

    /// 删除密码记录（移入回收站或永久删除）
    async fn delete_password(&self, id: &str, to_trash: bool) -> TuiResult<()> {
        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        if to_trash {
            vault.delete_record(id)
                .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;
        } else {
            vault.permanently_delete_record(id)
                .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;
        }
        Ok(())
    }

    /// 带过滤和排序的查询
    async fn query(&self, _request: ()) -> TuiResult<()> {
        // 当前 trait 参数和返回为 ()，暂时返回 Ok
        Ok(())
    }

    /// 获取各过滤条件的计数
    async fn get_filter_counts(&self) -> TuiResult<HashMap<String, usize>> {
        let vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        let records = vault.list_records()
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        let mut counts = HashMap::new();
        counts.insert("total".to_string(), records.len());

        Ok(counts)
    }

    /// 获取分组树
    async fn get_group_tree(&self) -> TuiResult<()> {
        // 当前 trait 返回 ()，暂时返回 Ok
        Ok(())
    }

    /// 获取单个分组
    async fn get_group(&self, _id: &str) -> TuiResult<()> {
        // 当前 trait 返回 ()，暂时返回 Ok
        Ok(())
    }

    /// 保存分组
    async fn save_group(&self, _group: &()) -> TuiResult<()> {
        // 当前 trait 参数为 ()，暂时返回 Ok
        Ok(())
    }

    /// 删除分组
    async fn delete_group(&self, _id: &str) -> TuiResult<()> {
        // 当前 trait 返回 ()，暂时返回 Ok
        Ok(())
    }

    /// 获取回收站中的项目
    async fn get_trash_items(&self) -> TuiResult<Vec<()>> {
        // For MVP, trash items are loaded via list_deleted_passwords() extension method
        Ok(Vec::new())
    }

    /// 从回收站恢复
    async fn restore_password(&self, id: &str) -> TuiResult<()> {
        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;
        vault.restore_record(id)
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;
        Ok(())
    }

    /// 永久删除
    async fn permanently_delete(&self, id: &str) -> TuiResult<()> {
        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;
        vault.permanently_delete_record(id)
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;
        Ok(())
    }

    /// 清空回收站
    async fn empty_trash(&self) -> TuiResult<usize> {
        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;
        vault.empty_trash()
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))
    }
}

// ============================================================================
// 扩展方法（非 trait 方法）
// ============================================================================

impl TuiDatabaseService {
    /// 获取所有密码记录（返回转换后的 PasswordRecord）
    pub async fn list_passwords(&self) -> TuiResult<Vec<PasswordRecord>> {
        let dek = self.dek.as_ref()
            .ok_or_else(|| TuiError::new(ErrorKind::InvalidKey))?;

        let vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        let records = vault.list_records()
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        let mut passwords = Vec::new();
        for record in records {
            if record.record_type == RecordType::Password {
                if let Ok(decrypted) = vault.get_record_decrypted(&record.id.to_string(), dek) {
                    passwords.push(Self::decrypted_to_password(&decrypted));
                }
            }
        }

        Ok(passwords)
    }

    /// Get all deleted (trashed) password records
    pub async fn list_deleted_passwords(&self) -> TuiResult<Vec<PasswordRecord>> {
        let dek = self.dek.as_ref()
            .ok_or_else(|| TuiError::new(ErrorKind::InvalidKey))?;

        let vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        let records = vault.list_deleted_records()
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        let mut passwords = Vec::new();
        for record in records {
            if record.record_type == RecordType::Password {
                // Decrypt the record using the stored record data directly
                // (get_record_decrypted filters by deleted=0, so we decrypt manually)
                let dek_array: [u8; 32] = *dek;
                if let Ok(decrypted_bytes) = crate::crypto::aes256gcm::decrypt(
                    &record.encrypted_data,
                    &record.nonce,
                    &dek_array,
                ) {
                    if let Ok(json_str) = String::from_utf8(decrypted_bytes) {
                        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            let decrypted = DecryptedRecord {
                                id: record.id,
                                record_type: record.record_type,
                                name: payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                username: payload.get("username").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                password: crate::types::SensitiveString::new(
                                    payload.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string()
                                ),
                                url: payload.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                notes: payload.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                tags: record.tags.clone(),
                                created_at: record.created_at,
                                updated_at: record.updated_at,
                                deleted: true,
                            };
                            passwords.push(Self::decrypted_to_password(&decrypted));
                        }
                    }
                }
            }
        }

        Ok(passwords)
    }

    /// 创建新密码记录
    pub async fn create_password(&self, password: &PasswordRecord) -> TuiResult<()> {
        let dek = self.dek.as_ref()
            .ok_or_else(|| TuiError::new(ErrorKind::InvalidKey))?;

        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        // Parse or generate UUID
        let id = uuid::Uuid::parse_str(&password.id)
            .unwrap_or_else(|_| uuid::Uuid::new_v4());

        // Create RecordPayload for encryption
        let payload = RecordPayload {
            name: password.name.clone(),
            username: password.username.clone(),
            password: password.password.clone(),
            url: password.url.clone(),
            notes: password.notes.clone(),
            tags: password.tags.clone(),
        };

        // Encrypt the payload using DEK
        let dek_array: [u8; 32] = *dek;

        let json_bytes = serde_json::to_vec(&payload)
            .map_err(|e| TuiError::new(ErrorKind::EncryptionFailed).with_details(e.to_string()))?;

        let (encrypted_data, nonce) = aes256gcm::encrypt(&json_bytes, &dek_array)
            .map_err(|e| TuiError::new(ErrorKind::EncryptionFailed).with_details(e.to_string()))?;

        // Create StoredRecord
        let record = StoredRecord {
            id,
            record_type: RecordType::Password,
            encrypted_data,
            nonce,
            tags: password.tags.clone(),
            created_at: password.created_at,
            updated_at: password.modified_at,
            version: 1,
            deleted: false,
        };

        // Store in vault
        vault.add_record(&record)
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        Ok(())
    }

    /// 更新密码记录
    pub async fn update_password(&self, password: &PasswordRecord) -> TuiResult<()> {
        let dek = self.dek.as_ref()
            .ok_or_else(|| TuiError::new(ErrorKind::InvalidKey))?;

        let mut vault = self.vault.lock()
            .map_err(|_| TuiError::new(ErrorKind::InvalidState("State lock failed".into())))?;

        // Parse UUID
        let id = uuid::Uuid::parse_str(&password.id)
            .map_err(|_| TuiError::new(ErrorKind::InvalidInput {
                field: "id".to_string(),
                reason: "Invalid UUID format".to_string(),
            }))?;

        // Get existing record to preserve created_at and version
        let existing = vault.get_record(&password.id)
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        // Create RecordPayload for encryption
        let payload = RecordPayload {
            name: password.name.clone(),
            username: password.username.clone(),
            password: password.password.clone(),
            url: password.url.clone(),
            notes: password.notes.clone(),
            tags: password.tags.clone(),
        };

        // Encrypt the payload using DEK
        let dek_array: [u8; 32] = *dek;

        let json_bytes = serde_json::to_vec(&payload)
            .map_err(|e| TuiError::new(ErrorKind::EncryptionFailed).with_details(e.to_string()))?;

        let (encrypted_data, nonce) = aes256gcm::encrypt(&json_bytes, &dek_array)
            .map_err(|e| TuiError::new(ErrorKind::EncryptionFailed).with_details(e.to_string()))?;

        // Create StoredRecord with updated data
        let record = StoredRecord {
            id,
            record_type: RecordType::Password,
            encrypted_data,
            nonce,
            tags: password.tags.clone(),
            created_at: existing.created_at, // Preserve original created_at
            updated_at: chrono::Utc::now(),
            version: existing.version, // Version will be incremented by vault.update_record
            deleted: false,
        };

        // Update in vault
        vault.update_record(&record)
            .map_err(|e| TuiError::new(ErrorKind::IoError(e.to_string())))?;

        Ok(())
    }
}

impl SecureClear for TuiDatabaseService {
    fn clear_sensitive_data(&mut self) {
        // 清除 DEK
        if let Some(mut dek) = self.dek.take() {
            zeroize::Zeroize::zeroize(&mut dek);
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_service_creation() {
        let service = TuiDatabaseService::new();
        assert!(service.get_filter_counts().await.is_ok());
    }

    #[tokio::test]
    async fn test_database_service_default() {
        let service = TuiDatabaseService::default();
        assert!(service.get_filter_counts().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_trash_items() {
        let service = TuiDatabaseService::new();
        let items = service.get_trash_items().await.unwrap();
        assert!(items.is_empty());
    }

    #[tokio::test]
    async fn test_empty_trash() {
        let service = TuiDatabaseService::new();
        let count = service.empty_trash().await.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_secure_clear() {
        let mut service = TuiDatabaseService::new();
        service.clear_sensitive_data();
        // Should not panic
    }

    #[test]
    fn test_is_initialized() {
        let service = TuiDatabaseService::new();
        assert!(!service.is_initialized());

        let service = service.with_dek([1u8; 32]);
        assert!(service.is_initialized());
    }

    #[test]
    fn test_decrypted_to_password() {
        let record = DecryptedRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Password,
            name: "Test Password".to_string(),
            username: Some("user@example.com".to_string()),
            password: crate::types::SensitiveString::new("secret123".to_string()),
            url: Some("https://example.com".to_string()),
            notes: Some("Test notes".to_string()),
            tags: vec!["work".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted: false,
        };

        let password = TuiDatabaseService::decrypted_to_password(&record);

        assert_eq!(password.name, "Test Password");
        assert_eq!(password.username, Some("user@example.com".to_string()));
        assert_eq!(password.password, "secret123");
        assert_eq!(password.url, Some("https://example.com".to_string()));
        assert_eq!(password.notes, Some("Test notes".to_string()));
        assert_eq!(password.tags, vec!["work"]);
    }
}
