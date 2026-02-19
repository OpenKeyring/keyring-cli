//! TUI 数据模型
//!
//! 定义 TUI 层使用的数据结构，包括密码记录、分组、树形结构等。

pub mod password;
pub mod group;
pub mod tree;

// TODO: 在实现各模块后取消注释以下导出
// pub use password::{PasswordRecord, PasswordFilter, PasswordType, SortField, SortOrder, QueryRequest, QueryResult};
// pub use group::{Group, GroupNodeData, Id, ParentId};
// pub use tree::{TreeNode, TreeNodeItem, TreeBuilder, GroupTree};
