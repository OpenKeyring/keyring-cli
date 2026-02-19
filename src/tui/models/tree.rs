//! 树形数据结构
//!
//! 提供通用的树形结构，用于分组树等场景。

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::models::group::{GroupNodeData, Id, ParentId};
use std::collections::HashMap;

/// 树节点
#[derive(Debug, Clone)]
pub struct TreeNode<T: Clone> {
    /// 节点 ID
    pub id: String,
    /// 层级深度
    pub level: u8,
    /// 节点项
    pub item: TreeNodeItem<T>,
    /// 子节点
    pub children: Vec<TreeNode<T>>,
    /// 是否展开
    pub expanded: bool,
    /// 父节点 ID
    pub parent_id: Option<String>,
}

/// 树节点项类型
#[derive(Debug, Clone)]
pub enum TreeNodeItem<T> {
    /// 文件夹节点
    Folder {
        /// 文件夹名称
        name: String,
        /// 子节点数量
        child_count: usize,
    },
    /// 数据节点
    Data(T),
}

impl<T: Clone> TreeNode<T> {
    /// 检查是否为文件夹节点
    pub fn is_folder(&self) -> bool {
        matches!(self.item, TreeNodeItem::Folder { .. })
    }

    /// 检查是否为数据节点
    pub fn is_data(&self) -> bool {
        matches!(self.item, TreeNodeItem::Data(_))
    }

    /// 获取节点名称
    pub fn name(&self) -> &str {
        match &self.item {
            TreeNodeItem::Folder { name, .. } => name,
            TreeNodeItem::Data(_) => &self.id,
        }
    }

    /// 获取子节点数量
    pub fn child_count(&self) -> usize {
        match &self.item {
            TreeNodeItem::Folder { child_count, .. } => *child_count,
            TreeNodeItem::Data(_) => 0,
        }
    }

    /// 切换展开状态
    pub fn toggle_expanded(&mut self) {
        self.expanded = !self.expanded;
    }

    /// 设置展开状态
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    /// 递归展开所有子节点
    pub fn expand_all(&mut self) {
        self.expanded = true;
        for child in &mut self.children {
            child.expand_all();
        }
    }

    /// 递归折叠所有子节点
    pub fn collapse_all(&mut self) {
        self.expanded = false;
        for child in &mut self.children {
            child.collapse_all();
        }
    }

    /// 查找节点
    pub fn find(&self, id: &str) -> Option<&TreeNode<T>> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    /// 查找可变节点
    pub fn find_mut(&mut self, id: &str) -> Option<&mut TreeNode<T>> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// 遍历所有节点（深度优先）
    pub fn iter_depth_first(&self) -> impl Iterator<Item = &TreeNode<T>> {
        TreeNodeIterator::new(self)
    }

    /// 获取所有后代节点数量
    pub fn descendant_count(&self) -> usize {
        self.children.iter().map(|c| 1 + c.descendant_count()).sum()
    }
}

/// 分组树类型
pub type GroupTree = TreeNode<GroupNodeData>;

/// 树构建器
#[derive(Debug, Clone)]
pub struct TreeBuilder {
    /// 最大层级深度
    max_level: u8,
}

impl TreeBuilder {
    /// 创建新的树构建器
    pub fn new(max_level: u8) -> Self {
        Self { max_level }
    }

    /// 从扁平列表构建树
    pub fn from_flat<T: Clone + Id + ParentId>(&self, nodes: Vec<T>) -> TuiResult<GroupTree> {
        // 按 parent_id 分组
        let mut children_map: HashMap<Option<String>, Vec<&T>> = HashMap::new();
        for node in &nodes {
            let parent = node.parent_id().map(|s| s.to_string());
            children_map.entry(parent).or_default().push(node);
        }

        // 找到根节点（parent_id 为 None）
        let root_nodes = children_map
            .get(&None)
            .ok_or_else(|| TuiError::invalid_state("No root nodes found"))?;

        if root_nodes.is_empty() {
            return Err(TuiError::invalid_state("Empty tree"));
        }

        // 使用第一个根节点构建树
        let first_root = root_nodes[0];
        self.build_node(first_root, 0, &children_map)
    }

    fn build_node<T: Clone + Id + ParentId>(
        &self,
        data: &T,
        level: u8,
        children_map: &HashMap<Option<String>, Vec<&T>>,
    ) -> TuiResult<GroupTree> {
        if level > self.max_level {
            return Err(TuiError::invalid_state(format!(
                "Tree level {} exceeds max level {}",
                level, self.max_level
            )));
        }

        let id = data.id().to_string();
        let parent_id = data.parent_id().map(|s| s.to_string());

        // 获取子节点
        let child_refs = children_map
            .get(&Some(id.clone()))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let child_count = child_refs.len();

        // 递归构建子节点
        let mut children = Vec::new();
        for child in child_refs {
            let child_node = self.build_node(*child, level + 1, children_map)?;
            children.push(child_node);
        }

        Ok(TreeNode {
            id: id.clone(),
            level,
            item: TreeNodeItem::Folder {
                name: id,
                child_count,
            },
            children,
            expanded: false,
            parent_id,
        })
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new(10) // 默认最大层级为 10
    }
}

/// 树节点迭代器（深度优先）
pub struct TreeNodeIterator<'a, T: Clone> {
    stack: Vec<&'a TreeNode<T>>,
}

impl<'a, T: Clone> TreeNodeIterator<'a, T> {
    fn new(root: &'a TreeNode<T>) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a, T: Clone> Iterator for TreeNodeIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // 将子节点逆序压入栈中，保证顺序正确
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node(id: &str, level: u8, parent_id: Option<&str>) -> GroupTree {
        TreeNode {
            id: id.to_string(),
            level,
            item: TreeNodeItem::Folder {
                name: id.to_string(),
                child_count: 0,
            },
            children: Vec::new(),
            expanded: false,
            parent_id: parent_id.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_tree_node_creation() {
        let node = create_test_node("root", 0, None);

        assert_eq!(node.id, "root");
        assert_eq!(node.level, 0);
        assert!(node.parent_id.is_none());
        assert!(!node.expanded);
        assert!(node.is_folder());
        assert!(!node.is_data());
    }

    #[test]
    fn test_tree_node_name() {
        let mut node = create_test_node("test-id", 0, None);
        assert_eq!(node.name(), "test-id");

        // 修改为 Data 节点测试 - 对于 Data 节点，name() 返回节点 id
        node.item = TreeNodeItem::Data(GroupNodeData::new("data-id", "Data Name", None));
        // 由于 TreeNode<T> 是泛型的，name() 对于 Data 项返回 self.id
        assert_eq!(node.name(), "test-id");
    }

    #[test]
    fn test_tree_node_child_count() {
        let folder_node: TreeNode<GroupNodeData> = TreeNode {
            id: "folder".to_string(),
            level: 0,
            item: TreeNodeItem::Folder {
                name: "Folder".to_string(),
                child_count: 5,
            },
            children: Vec::new(),
            expanded: false,
            parent_id: None,
        };
        assert_eq!(folder_node.child_count(), 5);

        let data_node = TreeNode::<GroupNodeData> {
            id: "data".to_string(),
            level: 0,
            item: TreeNodeItem::Data(GroupNodeData::new("data", "Data", None)),
            children: Vec::new(),
            expanded: false,
            parent_id: None,
        };
        assert_eq!(data_node.child_count(), 0);
    }

    #[test]
    fn test_tree_node_toggle_expanded() {
        let mut node = create_test_node("root", 0, None);
        assert!(!node.expanded);

        node.toggle_expanded();
        assert!(node.expanded);

        node.toggle_expanded();
        assert!(!node.expanded);
    }

    #[test]
    fn test_tree_node_set_expanded() {
        let mut node = create_test_node("root", 0, None);

        node.set_expanded(true);
        assert!(node.expanded);

        node.set_expanded(false);
        assert!(!node.expanded);
    }

    #[test]
    fn test_tree_node_expand_collapse_all() {
        let mut root = create_test_node("root", 0, None);
        let child1 = create_test_node("child1", 1, Some("root"));
        let grandchild = create_test_node("grandchild", 2, Some("child1"));

        root.children.push(child1);
        root.children[0].children.push(grandchild);

        root.expand_all();
        assert!(root.expanded);
        assert!(root.children[0].expanded);
        assert!(root.children[0].children[0].expanded);

        root.collapse_all();
        assert!(!root.expanded);
        assert!(!root.children[0].expanded);
        assert!(!root.children[0].children[0].expanded);
    }

    #[test]
    fn test_tree_node_find() {
        let mut root = create_test_node("root", 0, None);
        let child1 = create_test_node("child1", 1, Some("root"));
        let child2 = create_test_node("child2", 1, Some("root"));
        let grandchild = create_test_node("grandchild", 2, Some("child1"));

        root.children.push(child1);
        root.children.push(child2);
        root.children[0].children.push(grandchild);

        assert!(root.find("root").is_some());
        assert!(root.find("child1").is_some());
        assert!(root.find("child2").is_some());
        assert!(root.find("grandchild").is_some());
        assert!(root.find("not-exist").is_none());
    }

    #[test]
    fn test_tree_node_find_mut() {
        let mut root = create_test_node("root", 0, None);
        let child = create_test_node("child", 1, Some("root"));
        root.children.push(child);

        let found = root.find_mut("child");
        assert!(found.is_some());
        found.unwrap().expanded = true;

        assert!(root.children[0].expanded);
    }

    #[test]
    fn test_tree_node_descendant_count() {
        let mut root = create_test_node("root", 0, None);
        let child1 = create_test_node("child1", 1, Some("root"));
        let child2 = create_test_node("child2", 1, Some("root"));
        let grandchild = create_test_node("grandchild", 2, Some("child1"));

        root.children.push(child1);
        root.children.push(child2);
        root.children[0].children.push(grandchild);

        assert_eq!(root.descendant_count(), 3); // 2 children + 1 grandchild
        assert_eq!(root.children[0].descendant_count(), 1); // 1 grandchild
        assert_eq!(root.children[1].descendant_count(), 0);
    }

    #[test]
    fn test_tree_node_iter_depth_first() {
        let mut root = create_test_node("root", 0, None);
        let child1 = create_test_node("child1", 1, Some("root"));
        let child2 = create_test_node("child2", 1, Some("root"));

        root.children.push(child1);
        root.children.push(child2);

        let ids: Vec<&str> = root.iter_depth_first().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["root", "child1", "child2"]);
    }

    #[test]
    fn test_tree_builder_from_flat() {
        let items = vec![
            GroupNodeData::new("1", "Root", None),
            GroupNodeData::new("2", "Child", Some("1".to_string())),
            GroupNodeData::new("3", "Grandchild", Some("2".to_string())),
        ];

        let builder = TreeBuilder::new(3);
        let tree = builder.from_flat(items).unwrap();

        assert_eq!(tree.id, "1");
        assert_eq!(tree.level, 0);
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].id, "2");
        assert_eq!(tree.children[0].level, 1);
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(tree.children[0].children[0].id, "3");
        assert_eq!(tree.children[0].children[0].level, 2);
    }

    #[test]
    fn test_tree_builder_multiple_children() {
        let items = vec![
            GroupNodeData::new("root", "Root", None),
            GroupNodeData::new("child1", "Child 1", Some("root".to_string())),
            GroupNodeData::new("child2", "Child 2", Some("root".to_string())),
            GroupNodeData::new("child3", "Child 3", Some("root".to_string())),
        ];

        let builder = TreeBuilder::new(3);
        let tree = builder.from_flat(items).unwrap();

        assert_eq!(tree.id, "root");
        assert_eq!(tree.children.len(), 3);
        assert_eq!(tree.child_count(), 3);
    }

    #[test]
    fn test_tree_builder_empty() {
        let items: Vec<GroupNodeData> = vec![];

        let builder = TreeBuilder::new(3);
        let result = builder.from_flat(items);

        assert!(result.is_err());
    }

    #[test]
    fn test_tree_builder_max_level_exceeded() {
        // 创建深度为 5 的树，但 max_level 为 2
        let items = vec![
            GroupNodeData::new("1", "L1", None),
            GroupNodeData::new("2", "L2", Some("1".to_string())),
            GroupNodeData::new("3", "L3", Some("2".to_string())),
            GroupNodeData::new("4", "L4", Some("3".to_string())),
            GroupNodeData::new("5", "L5", Some("4".to_string())),
        ];

        let builder = TreeBuilder::new(2);
        let result = builder.from_flat(items);

        assert!(result.is_err());
    }

    #[test]
    fn test_tree_builder_default() {
        let builder = TreeBuilder::default();
        assert_eq!(builder.max_level, 10);
    }

    #[test]
    fn test_tree_node_item_folder() {
        let item: TreeNodeItem<GroupNodeData> = TreeNodeItem::Folder {
            name: "Test Folder".to_string(),
            child_count: 5,
        };

        if let TreeNodeItem::Folder { name, child_count } = item {
            assert_eq!(name, "Test Folder");
            assert_eq!(child_count, 5);
        } else {
            panic!("Expected Folder variant");
        }
    }

    #[test]
    fn test_tree_node_item_data() {
        let data = GroupNodeData::new("id", "name", None);
        let item: TreeNodeItem<GroupNodeData> = TreeNodeItem::Data(data.clone());

        if let TreeNodeItem::Data(d) = item {
            assert_eq!(d.id, "id");
            assert_eq!(d.name, "name");
        } else {
            panic!("Expected Data variant");
        }
    }
}
