//! Tests for TreePanel component
//!
//! Unit tests for the tree panel component.

use super::*;
use crate::tui::state::{AppState, NodeType, TreeNodeId, VisibleNode};
use crate::tui::traits::{Component, Interactive};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

fn create_visible_node(
    id: TreeNodeId,
    level: u8,
    node_type: NodeType,
    label: &str,
) -> VisibleNode {
    VisibleNode {
        id,
        level,
        node_type,
        label: label.to_string(),
        child_count: 0,
    }
}

#[test]
fn test_tree_panel_creation() {
    let panel = TreePanel::new();
    assert!(!panel.focused);
    assert!(panel.can_focus());
}

#[test]
fn test_focus_state() {
    let mut panel = TreePanel::new();
    assert!(!panel.is_focused());

    panel.on_focus_gain().unwrap();
    assert!(panel.is_focused());

    panel.on_focus_loss().unwrap();
    assert!(!panel.is_focused());
}

#[test]
fn test_navigation_j_k() {
    let mut panel = TreePanel::new();
    let mut state = AppState::new();

    // Create some test nodes
    state.tree.set_visible_nodes(vec![
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Root"),
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Child 1"),
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Child 2"),
    ]);

    // Initial index should be 0
    assert_eq!(state.tree.highlighted_index, 0);

    // Press 'j' to move down
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.tree.highlighted_index, 1);

    // Press 'k' to move up
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.tree.highlighted_index, 0);
}

#[test]
fn test_navigation_g_g() {
    let mut panel = TreePanel::new();
    let mut state = AppState::new();

    // Create some test nodes
    state.tree.set_visible_nodes(vec![
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Root"),
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Child 1"),
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Child 2"),
    ]);
    state.tree.highlighted_index = 1;

    // Press 'g' once - should NOT move yet, just set pending_g
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    // Single 'g' should not move, index stays at same
    assert_eq!(state.tree.highlighted_index, 1);
    assert!(panel.pending_g);

    // Press 'g' again quickly - should move to top (gg sequence)
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.tree.highlighted_index, 0); // Now at top
    assert!(!panel.pending_g); // pending_g cleared

    // Move to middle for next test
    state.tree.highlighted_index = 1;

    // Press 'G' (Shift+g) to move to bottom
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.tree.highlighted_index, 2);
}

#[test]
fn test_expand_collapse_folder() {
    use crate::tui::models::password::PasswordRecord;

    let mut panel = TreePanel::new();
    let mut state = AppState::new();

    // Create test data
    let group_id = Uuid::new_v4();
    let password_id = Uuid::new_v4();
    let test_password = PasswordRecord::new(password_id.to_string(), "Test Password", "secret123")
        .with_group(group_id.to_string());

    // Add to cache so apply_filter produces data
    state.refresh_password_cache(vec![test_password]);

    // Manually set visible nodes with a group for testing expand/collapse
    state.tree.set_visible_nodes(vec![
        create_visible_node(
            TreeNodeId::Group(group_id),
            0,
            NodeType::Folder,
            "Test Group",
        ),
        create_visible_node(
            TreeNodeId::Password(password_id),
            1,
            NodeType::Password,
            "Test Password",
        ),
    ]);

    // Get the group ID from visible nodes
    let test_group_id = match state.tree.current_node() {
        Some(node) => {
            if let TreeNodeId::Group(id) = node.id {
                id
            } else {
                panic!("Expected first node to be a group");
            }
        }
        None => panic!("No visible nodes available"),
    };

    // Initially not expanded
    assert!(!state.tree.is_expanded(&test_group_id));

    // Press 'l' to expand
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert!(state.tree.is_expanded(&test_group_id));

    // Reset visible nodes (simulate re-rendering after expand)
    state.tree.set_visible_nodes(vec![
        create_visible_node(
            TreeNodeId::Group(group_id),
            0,
            NodeType::Folder,
            "Test Group",
        ),
        create_visible_node(
            TreeNodeId::Password(password_id),
            1,
            NodeType::Password,
            "Test Password",
        ),
    ]);
    state.tree.highlighted_index = 0; // Keep focus on group

    // Press 'h' to collapse
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert!(!state.tree.is_expanded(&test_group_id));
}

#[test]
fn test_select_password() {
    let mut panel = TreePanel::new();
    let mut state = AppState::new();

    let password_id = Uuid::new_v4();
    state.tree.set_visible_nodes(vec![
        create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Root"),
        create_visible_node(
            TreeNodeId::Password(password_id),
            1,
            NodeType::Password,
            "Entry",
        ),
    ]);
    state.tree.highlighted_index = 1;

    // Initially no selection
    assert!(state.selection.selected_password.is_none());

    // Press Enter to select
    let result = panel.handle_key_with_state(
        KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        &mut state,
    );
    assert!(matches!(result, HandleResult::Consumed));
    assert_eq!(state.selection.selected_password, Some(password_id));
}

#[test]
fn test_get_selected_password() {
    let panel = TreePanel::new();
    let mut state = AppState::new();

    let password_id = Uuid::new_v4();
    state.tree.set_visible_nodes(vec![create_visible_node(
        TreeNodeId::Password(password_id),
        0,
        NodeType::Password,
        "Entry",
    )]);

    let selected = panel.get_selected_password(&state.tree);
    assert_eq!(selected, Some(password_id));
}

#[test]
fn test_get_selected_password_when_folder() {
    let panel = TreePanel::new();
    let mut state = AppState::new();

    let group_id = Uuid::new_v4();
    state.tree.set_visible_nodes(vec![create_visible_node(
        TreeNodeId::Group(group_id),
        0,
        NodeType::Folder,
        "Folder",
    )]);

    let selected = panel.get_selected_password(&state.tree);
    assert!(selected.is_none());
}
