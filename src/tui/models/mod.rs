//! TUI 数据模型
//!
//! 定义 TUI 层使用的数据结构，包括密码记录、分组、树形结构等。

pub mod group;
pub mod password;
pub mod tree;

pub use group::{Group, GroupNodeData, Id, ParentId};
pub use password::{
    PasswordFilter, PasswordRecord, PasswordType, QueryRequest, QueryResult, SortField, SortOrder,
};
pub use tree::{GroupTree, TreeBuilder, TreeNode, TreeNodeItem};
