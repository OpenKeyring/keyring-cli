//! 分组和分组节点数据类型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 分组记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// 分组 ID
    pub id: String,
    /// 分组名称
    pub name: String,
    /// 父分组 ID（None 表示根分组）
    pub parent_id: Option<String>,
    /// 分组层级（0 表示根）
    pub level: u8,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 分组节点数据（用于树形结构）
#[derive(Debug, Clone)]
pub struct GroupNodeData {
    /// 节点 ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 父节点 ID
    pub parent_id: Option<String>,
}

impl GroupNodeData {
    /// 创建新的分组节点数据
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, parent_id: Option<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent_id,
        }
    }
}

/// Id trait - 用于树构建器
pub trait Id {
    /// 获取 ID
    fn id(&self) -> &str;
}

/// ParentId trait - 用于树构建器
pub trait ParentId {
    /// 获取父 ID
    fn parent_id(&self) -> Option<&str>;
}

impl Id for GroupNodeData {
    fn id(&self) -> &str {
        &self.id
    }
}

impl ParentId for GroupNodeData {
    fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }
}

impl Id for Group {
    fn id(&self) -> &str {
        &self.id
    }
}

impl ParentId for Group {
    fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_group_creation() {
        let group = Group {
            id: "group-1".to_string(),
            name: "Work".to_string(),
            parent_id: None,
            level: 0,
            created_at: Utc::now(),
        };

        assert_eq!(group.id, "group-1");
        assert_eq!(group.name, "Work");
        assert!(group.parent_id.is_none());
        assert_eq!(group.level, 0);
    }

    #[test]
    fn test_group_with_parent() {
        let group = Group {
            id: "group-2".to_string(),
            name: "Email Accounts".to_string(),
            parent_id: Some("group-1".to_string()),
            level: 1,
            created_at: Utc::now(),
        };

        assert_eq!(group.id, "group-2");
        assert_eq!(group.name, "Email Accounts");
        assert_eq!(group.parent_id, Some("group-1".to_string()));
        assert_eq!(group.level, 1);
    }

    #[test]
    fn test_group_node_data_creation() {
        let data = GroupNodeData {
            id: "node-1".to_string(),
            name: "Personal".to_string(),
            parent_id: Some("root".to_string()),
        };

        assert_eq!(data.id, "node-1");
        assert_eq!(data.name, "Personal");
        assert_eq!(data.parent_id, Some("root".to_string()));
    }

    #[test]
    fn test_group_node_data_new() {
        let data = GroupNodeData::new("node-2", "Work", Some("root".to_string()));

        assert_eq!(data.id, "node-2");
        assert_eq!(data.name, "Work");
        assert_eq!(data.parent_id, Some("root".to_string()));
    }

    #[test]
    fn test_group_node_data_root() {
        let data = GroupNodeData::new("root", "Root Group", None);

        assert_eq!(data.id, "root");
        assert_eq!(data.name, "Root Group");
        assert!(data.parent_id.is_none());
    }

    #[test]
    fn test_id_trait_for_group_node_data() {
        let data = GroupNodeData {
            id: "node-1".to_string(),
            name: "Personal".to_string(),
            parent_id: Some("root".to_string()),
        };

        assert_eq!(data.id(), "node-1");
    }

    #[test]
    fn test_parent_id_trait_for_group_node_data() {
        let data_with_parent = GroupNodeData {
            id: "node-1".to_string(),
            name: "Personal".to_string(),
            parent_id: Some("root".to_string()),
        };

        let data_without_parent = GroupNodeData {
            id: "root".to_string(),
            name: "Root".to_string(),
            parent_id: None,
        };

        assert_eq!(data_with_parent.parent_id(), Some("root"));
        assert_eq!(data_without_parent.parent_id(), None);
    }

    #[test]
    fn test_id_trait_for_group() {
        let group = Group {
            id: "group-123".to_string(),
            name: "Test Group".to_string(),
            parent_id: None,
            level: 0,
            created_at: Utc::now(),
        };

        assert_eq!(group.id(), "group-123");
    }

    #[test]
    fn test_parent_id_trait_for_group() {
        let group_with_parent = Group {
            id: "child".to_string(),
            name: "Child".to_string(),
            parent_id: Some("parent".to_string()),
            level: 1,
            created_at: Utc::now(),
        };

        let group_without_parent = Group {
            id: "root".to_string(),
            name: "Root".to_string(),
            parent_id: None,
            level: 0,
            created_at: Utc::now(),
        };

        assert_eq!(group_with_parent.parent_id(), Some("parent"));
        assert_eq!(group_without_parent.parent_id(), None);
    }

    #[test]
    fn test_group_serialization() {
        let group = Group {
            id: "g1".to_string(),
            name: "Test".to_string(),
            parent_id: None,
            level: 0,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("\"id\":\"g1\""));
        assert!(json.contains("\"name\":\"Test\""));
    }

    #[test]
    fn test_group_deserialization() {
        let json = r#"{
            "id": "g1",
            "name": "Test Group",
            "parent_id": "parent",
            "level": 1,
            "created_at": "2024-01-01T00:00:00Z"
        }"#;

        let group: Group = serde_json::from_str(json).unwrap();
        assert_eq!(group.id, "g1");
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.parent_id, Some("parent".to_string()));
        assert_eq!(group.level, 1);
    }
}
