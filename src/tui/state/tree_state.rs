//! Tree component state management

use std::collections::HashSet;
use uuid::Uuid;

/// Tree node ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeNodeId {
    /// Group node
    Group(Uuid),
    /// Password node
    Password(Uuid),
}

/// Node type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Folder (group)
    Folder,
    /// Password item
    Password,
}

/// Visible node (flattened representation)
#[derive(Debug, Clone)]
pub struct VisibleNode {
    /// Node ID
    pub id: TreeNodeId,
    /// Indent level 0-2 (max 3 levels)
    pub level: u8,
    /// Node type
    pub node_type: NodeType,
    /// Display label
    pub label: String,
    /// Child count
    pub child_count: usize,
    /// Whether this password is marked as favorite (only for Password nodes)
    pub is_favorite: bool,
}

/// Tree component state
#[derive(Debug, Clone, Default)]
pub struct TreeState {
    /// Currently expanded group IDs
    pub expanded_groups: HashSet<Uuid>,
    /// Current highlighted node index
    pub highlighted_index: usize,
    /// Current highlighted node
    pub highlighted_node: Option<TreeNodeId>,
    /// Tree node list (flattened, with indent levels)
    pub visible_nodes: Vec<VisibleNode>,
}

impl TreeState {
    /// Create new tree state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle expand/collapse for a group
    pub fn toggle_expand(&mut self, group_id: Uuid) {
        if self.expanded_groups.contains(&group_id) {
            self.expanded_groups.remove(&group_id);
        } else {
            self.expanded_groups.insert(group_id);
        }
    }

    /// Check if a group is expanded
    pub fn is_expanded(&self, group_id: &Uuid) -> bool {
        self.expanded_groups.contains(group_id)
    }

    /// Move up (vim k style)
    pub fn move_up(&mut self) {
        if self.visible_nodes.is_empty() {
            return;
        }
        if self.highlighted_index > 0 {
            self.highlighted_index -= 1;
        } else {
            // Wrap to end
            self.highlighted_index = self.visible_nodes.len() - 1;
        }
        self.update_highlighted_node();
    }

    /// Move down (vim j style)
    pub fn move_down(&mut self) {
        if self.visible_nodes.is_empty() {
            return;
        }
        if self.highlighted_index < self.visible_nodes.len() - 1 {
            self.highlighted_index += 1;
        } else {
            // Wrap to beginning
            self.highlighted_index = 0;
        }
        self.update_highlighted_node();
    }

    /// Move to top (vim g style)
    pub fn move_to_top(&mut self) {
        self.highlighted_index = 0;
        self.update_highlighted_node();
    }

    /// Move to bottom (vim G style)
    pub fn move_to_bottom(&mut self) {
        if !self.visible_nodes.is_empty() {
            self.highlighted_index = self.visible_nodes.len() - 1;
            self.update_highlighted_node();
        }
    }

    /// Update highlighted node reference
    fn update_highlighted_node(&mut self) {
        self.highlighted_node = self.visible_nodes.get(self.highlighted_index).map(|n| n.id);
    }

    /// Get current highlighted node
    pub fn current_node(&self) -> Option<&VisibleNode> {
        self.visible_nodes.get(self.highlighted_index)
    }

    /// Set visible nodes list
    pub fn set_visible_nodes(&mut self, nodes: Vec<VisibleNode>) {
        self.visible_nodes = nodes;
        self.update_highlighted_node();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_visible_node(id: TreeNodeId) -> VisibleNode {
        VisibleNode {
            id,
            level: 0,
            node_type: NodeType::Folder,
            label: "Test".to_string(),
            child_count: 0,
            is_favorite: false,
        }
    }

    #[test]
    fn test_tree_state_default() {
        let state = TreeState::default();
        assert!(state.expanded_groups.is_empty());
        assert!(state.highlighted_node.is_none());
        assert!(state.visible_nodes.is_empty());
    }

    #[test]
    fn test_toggle_expand() {
        let mut state = TreeState::default();
        let group_id = Uuid::new_v4();

        state.toggle_expand(group_id);
        assert!(state.expanded_groups.contains(&group_id));

        state.toggle_expand(group_id);
        assert!(!state.expanded_groups.contains(&group_id));
    }

    #[test]
    fn test_move_up_down() {
        let mut state = TreeState {
            visible_nodes: vec![
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
            ],
            highlighted_index: 1,
            ..Default::default()
        };

        state.move_up();
        assert_eq!(state.highlighted_index, 0);

        state.move_down();
        assert_eq!(state.highlighted_index, 1);

        // Boundary test - wrap around
        state.highlighted_index = 0;
        state.move_up(); // Wrap to end
        assert_eq!(state.highlighted_index, 2);

        state.highlighted_index = 2;
        state.move_down(); // Wrap to beginning
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_move_to_top_bottom() {
        let mut state = TreeState {
            visible_nodes: vec![
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
                create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
            ],
            highlighted_index: 1,
            ..Default::default()
        };

        state.move_to_top();
        assert_eq!(state.highlighted_index, 0);

        state.move_to_bottom();
        assert_eq!(state.highlighted_index, 2);
    }

    #[test]
    fn test_current_node() {
        let mut state = TreeState::default();
        let node = create_visible_node(TreeNodeId::Group(Uuid::new_v4()));
        state.visible_nodes = vec![node.clone()];
        state.highlighted_index = 0;

        let current = state.current_node();
        assert!(current.is_some());
        assert_eq!(current.unwrap().label, "Test");
    }

    #[test]
    fn test_set_visible_nodes() {
        let mut state = TreeState::default();
        let nodes = vec![
            create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
            create_visible_node(TreeNodeId::Group(Uuid::new_v4())),
        ];

        state.set_visible_nodes(nodes);
        assert_eq!(state.visible_nodes.len(), 2);
        assert!(state.highlighted_node.is_some());
    }

    #[test]
    fn test_is_expanded() {
        let mut state = TreeState::default();
        let group_id = Uuid::new_v4();

        assert!(!state.is_expanded(&group_id));

        state.toggle_expand(group_id);
        assert!(state.is_expanded(&group_id));
    }
}
