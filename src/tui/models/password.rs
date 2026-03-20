//! 密码记录和查询类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 密码记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRecord {
    /// 记录 ID
    pub id: String,
    /// 记录名称
    pub name: String,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: String,
    /// URL
    pub url: Option<String>,
    /// 备注
    pub notes: Option<String>,
    /// 标签列表
    pub tags: Vec<String>,
    /// 分组 ID
    pub group_id: Option<String>,
    /// 是否收藏
    pub is_favorite: bool,
    /// 是否已删除（在回收站中）
    pub is_deleted: bool,
    /// 删除时间
    pub deleted_at: Option<DateTime<Utc>>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 修改时间
    pub modified_at: DateTime<Utc>,
}

impl PasswordRecord {
    /// 创建新的密码记录
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            username: None,
            password: password.into(),
            url: None,
            notes: None,
            tags: Vec::new(),
            group_id: None,
            is_favorite: false,
            is_deleted: false,
            deleted_at: None,
            expires_at: None,
            created_at: now,
            modified_at: now,
        }
    }

    /// 设置用户名
    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// 设置 URL
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// 设置备注
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// 设置标签
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// 设置分组 ID
    #[must_use]
    pub fn with_group(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }

    /// 设置为收藏
    #[must_use]
    pub fn with_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }
}

/// 密码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PasswordType {
    /// PIN 码
    Pin,
    /// 随机密码
    Random,
    /// 易记密码
    Memorable,
}

impl Default for PasswordType {
    fn default() -> Self {
        Self::Random
    }
}

/// 查询过滤器
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PasswordFilter {
    /// 全部
    All,
    /// 回收站
    Trash,
    /// 收藏
    Favorites,
    /// 已过期
    Expired,
    /// 按标签过滤
    Tag(String),
    /// 按分组过滤
    Group(String),
    /// 搜索
    Search(String),
}

impl Default for PasswordFilter {
    fn default() -> Self {
        Self::All
    }
}

/// 排序字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    /// 按名称排序
    Name,
    /// 按创建时间排序
    CreatedAt,
    /// 按修改时间排序
    ModifiedAt,
    /// 按过期时间排序
    ExpiresAt,
}

impl Default for SortField {
    fn default() -> Self {
        Self::Name
    }
}

/// 排序顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// 升序
    Asc,
    /// 降序
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Asc
    }
}

/// 查询请求
#[derive(Debug, Clone)]
pub struct QueryRequest {
    /// 过滤条件
    pub filter: PasswordFilter,
    /// 排序字段
    pub sort_by: SortField,
    /// 排序顺序
    pub sort_order: SortOrder,
    /// 返回数量限制
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
}

impl Default for QueryRequest {
    fn default() -> Self {
        Self {
            filter: PasswordFilter::All,
            sort_by: SortField::Name,
            sort_order: SortOrder::Asc,
            limit: None,
            offset: None,
        }
    }
}

impl QueryRequest {
    /// 创建新的查询请求
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置过滤条件
    #[must_use]
    pub fn with_filter(mut self, filter: PasswordFilter) -> Self {
        self.filter = filter;
        self
    }

    /// 设置排序
    #[must_use]
    pub fn with_sort(mut self, sort_by: SortField, sort_order: SortOrder) -> Self {
        self.sort_by = sort_by;
        self.sort_order = sort_order;
        self
    }

    /// 设置分页
    #[must_use]
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 查询结果列表
    pub items: Vec<PasswordRecord>,
    /// 总数
    pub total_count: usize,
    /// 各过滤条件的计数
    pub filter_counts: HashMap<PasswordFilter, usize>,
}

impl QueryResult {
    /// 创建空的查询结果
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total_count: 0,
            filter_counts: HashMap::new(),
        }
    }

    /// 创建包含结果的查询结果
    pub fn new(items: Vec<PasswordRecord>, total_count: usize) -> Self {
        Self {
            items,
            total_count,
            filter_counts: HashMap::new(),
        }
    }

    /// 设置过滤计数
    #[must_use]
    pub fn with_filter_counts(mut self, counts: HashMap<PasswordFilter, usize>) -> Self {
        self.filter_counts = counts;
        self
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 获取结果数量
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_record_creation() {
        let record = PasswordRecord::new("test-123", "Test Password", "secret")
            .with_username("user@example.com")
            .with_url("https://example.com")
            .with_tags(vec!["work".to_string()]);

        assert_eq!(record.id, "test-123");
        assert_eq!(record.name, "Test Password");
        assert_eq!(record.password, "secret");
        assert!(record.username.is_some());
        assert_eq!(record.username.as_deref(), Some("user@example.com"));
        assert_eq!(record.url.as_deref(), Some("https://example.com"));
        assert_eq!(record.tags, vec!["work"]);
        assert!(!record.is_favorite);
        assert!(!record.is_deleted);
    }

    #[test]
    fn test_password_record_defaults() {
        let record = PasswordRecord::new("id", "name", "pass");

        assert!(record.username.is_none());
        assert!(record.url.is_none());
        assert!(record.notes.is_none());
        assert!(record.tags.is_empty());
        assert!(record.group_id.is_none());
        assert!(!record.is_favorite);
        assert!(!record.is_deleted);
        assert!(record.deleted_at.is_none());
        assert!(record.expires_at.is_none());
    }

    #[test]
    fn test_password_type_default() {
        let pt = PasswordType::default();
        assert_eq!(pt, PasswordType::Random);
    }

    #[test]
    fn test_password_filter_equality() {
        assert_eq!(PasswordFilter::All, PasswordFilter::All);
        assert_eq!(
            PasswordFilter::Tag("work".to_string()),
            PasswordFilter::Tag("work".to_string())
        );
        assert_ne!(PasswordFilter::All, PasswordFilter::Trash);
        assert_ne!(
            PasswordFilter::Tag("work".to_string()),
            PasswordFilter::Tag("personal".to_string())
        );
    }

    #[test]
    fn test_password_filter_default() {
        let filter = PasswordFilter::default();
        assert_eq!(filter, PasswordFilter::All);
    }

    #[test]
    fn test_sort_field_default() {
        let sort = SortField::default();
        assert_eq!(sort, SortField::Name);
    }

    #[test]
    fn test_sort_order_default() {
        let order = SortOrder::default();
        assert_eq!(order, SortOrder::Asc);
    }

    #[test]
    fn test_query_request_default() {
        let req = QueryRequest::default();
        assert_eq!(req.filter, PasswordFilter::All);
        assert_eq!(req.sort_by, SortField::Name);
        assert_eq!(req.sort_order, SortOrder::Asc);
        assert!(req.limit.is_none());
        assert!(req.offset.is_none());
    }

    #[test]
    fn test_query_request_builder() {
        let req = QueryRequest::new()
            .with_filter(PasswordFilter::Favorites)
            .with_sort(SortField::CreatedAt, SortOrder::Desc)
            .with_pagination(10, 20);

        assert_eq!(req.filter, PasswordFilter::Favorites);
        assert_eq!(req.sort_by, SortField::CreatedAt);
        assert_eq!(req.sort_order, SortOrder::Desc);
        assert_eq!(req.limit, Some(10));
        assert_eq!(req.offset, Some(20));
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);
        assert_eq!(result.total_count, 0);
        assert!(result.filter_counts.is_empty());
    }

    #[test]
    fn test_query_result_new() {
        let records = vec![
            PasswordRecord::new("1", "First", "pass1"),
            PasswordRecord::new("2", "Second", "pass2"),
        ];
        let result = QueryResult::new(records.clone(), 2);

        assert!(!result.is_empty());
        assert_eq!(result.len(), 2);
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_query_result_with_filter_counts() {
        let mut counts = HashMap::new();
        counts.insert(PasswordFilter::All, 10);
        counts.insert(PasswordFilter::Favorites, 3);

        let result = QueryResult::empty().with_filter_counts(counts);

        assert_eq!(result.filter_counts.get(&PasswordFilter::All), Some(&10));
        assert_eq!(
            result.filter_counts.get(&PasswordFilter::Favorites),
            Some(&3)
        );
    }
}
