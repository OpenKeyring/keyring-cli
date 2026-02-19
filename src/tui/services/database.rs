//! TUI 数据库服务适配器
//!
//! 封装现有 Vault 实现，提供 TUI 层所需的 DatabaseService trait。

use crate::tui::error::TuiResult;
use crate::tui::traits::{DatabaseService, SecureClear};
use async_trait::async_trait;
use std::collections::HashMap;

/// TUI 数据库服务
///
/// 适配现有的 Vault 实现，为 TUI 层提供统一的数据库访问接口。
pub struct TuiDatabaseService {
    // TODO: 添加 Vault 引用
    // vault: Arc<Vault>,
}

impl TuiDatabaseService {
    /// 创建新的数据库服务
    pub fn new() -> Self {
        Self {}
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
        // TODO: 调用 Vault::get_record_decrypted
        todo!("Implement with Vault integration")
    }

    /// 保存密码记录
    async fn save_password(&self, _record: &()) -> TuiResult<()> {
        // TODO: 调用 Vault::save_record
        todo!("Implement with Vault integration")
    }

    /// 删除密码记录（移入回收站或永久删除）
    async fn delete_password(&self, _id: &str, _to_trash: bool) -> TuiResult<()> {
        // TODO: 调用 Vault::delete_record
        todo!("Implement with Vault integration")
    }

    /// 带过滤和排序的查询
    async fn query(&self, _request: ()) -> TuiResult<()> {
        // TODO: 实现查询逻辑
        todo!("Implement with Vault integration")
    }

    /// 获取各过滤条件的计数
    async fn get_filter_counts(&self) -> TuiResult<HashMap<String, usize>> {
        // TODO: 实现过滤器计数
        Ok(HashMap::new())
    }

    /// 获取分组树
    async fn get_group_tree(&self) -> TuiResult<()> {
        // TODO: 调用 Vault 获取分组树
        todo!("Implement with Vault integration")
    }

    /// 获取单个分组
    async fn get_group(&self, _id: &str) -> TuiResult<()> {
        todo!("Implement with Vault integration")
    }

    /// 保存分组
    async fn save_group(&self, _group: &()) -> TuiResult<()> {
        todo!("Implement with Vault integration")
    }

    /// 删除分组
    async fn delete_group(&self, _id: &str) -> TuiResult<()> {
        todo!("Implement with Vault integration")
    }

    /// 获取回收站中的项目
    async fn get_trash_items(&self) -> TuiResult<Vec<()>> {
        // TODO: 调用 Vault::list_deleted_records
        Ok(Vec::new())
    }

    /// 从回收站恢复
    async fn restore_password(&self, _id: &str) -> TuiResult<()> {
        todo!("Implement with Vault integration")
    }

    /// 永久删除
    async fn permanently_delete(&self, _id: &str) -> TuiResult<()> {
        todo!("Implement with Vault integration")
    }

    /// 清空回收站
    async fn empty_trash(&self) -> TuiResult<usize> {
        Ok(0)
    }
}

impl SecureClear for TuiDatabaseService {
    fn clear_sensitive_data(&mut self) {
        // 无敏感数据需要清除
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
}
