//! Tree Panel Component
//!
//! Displays a tree view of password groups and entries.
//! Supports Vim-style navigation (j/k/g/G/gg) and expand/collapse (h/l).

use crate::tui::error::TuiResult;
use crate::tui::state::{AppState, TreeState, TreeNodeId, NodeType};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::Instant;
use uuid::Uuid;

/// Maximum time between gg key presses (in milliseconds)
const GG_TIMEOUT_MS: u128 = 500;

/// Tree panel component
///
/// Displays a hierarchical tree of groups and password entries.
/// Uses TreeState for navigation state, tracks gg double-key internally.
pub struct TreePanel {
    /// Component ID
    id: ComponentId,
    /// Whether the panel has focus
    focused: bool,
    /// Track pending 'g' key for gg double-key sequence
    pending_g: bool,
    /// Time of last 'g' key press
    last_g_time: Option<Instant>,
}

impl TreePanel {
    /// Create a new tree panel
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            focused: false,
            pending_g: false,
            last_g_time: None,
        }
    }

    /// Set component ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// Check if the panel is currently focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Render to frame (preferred method)
    ///
    /// Renders the tree using TreeState's visible_nodes list.
    /// Shows folder icons, expand/collapse indicators, and selection highlight.
    pub fn render_frame(&self, frame: &mut Frame, area: Rect, state: &TreeState) {
        if area.height < 3 {
            // Not enough space to render
            return;
        }

        // Create border block with focus-aware styling
        let border_style = if self.focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" [1] Groups ");

        let inner_area = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Render visible nodes
        self.render_nodes(frame, inner_area, state);
    }

    /// Render the list of visible nodes
    fn render_nodes(&self, frame: &mut Frame, area: Rect, state: &TreeState) {
        if state.visible_nodes.is_empty() {
            // Show empty state
            let empty_text = Paragraph::new("No entries")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty_text, area);
            return;
        }

        // Calculate how many rows we can display
        let max_rows = area.height as usize;
        let start_row = self.calculate_scroll_offset(state, max_rows);

        // Render visible nodes
        for (i, node) in state.visible_nodes.iter().skip(start_row).enumerate() {
            if i >= max_rows {
                break;
            }

            let y = area.y + i as u16;
            let row_area = Rect::new(area.x, y, area.width, 1);

            let is_highlighted = (start_row + i) == state.highlighted_index && self.focused;
            let is_expanded = if let TreeNodeId::Group(id) = node.id {
                state.is_expanded(&id)
            } else {
                false
            };

            let line = self.format_node_line(node, is_highlighted, is_expanded);
            let paragraph = Paragraph::new(line);
            paragraph.render(row_area, frame.buffer_mut());
        }
    }

    /// Calculate scroll offset to keep highlighted item visible
    fn calculate_scroll_offset(&self, state: &TreeState, max_rows: usize) -> usize {
        if state.highlighted_index < max_rows.saturating_sub(1) {
            return 0;
        }
        state.highlighted_index.saturating_sub(max_rows.saturating_sub(2))
    }

    /// Pre-computed indentation strings for tree levels
    const INDENTS: [&str; 10] = ["", "  ", "    ", "      ", "        ", "          ", "            ", "              ", "                ", "                  "];

    /// Format a single node line for display
    fn format_node_line(&self, node: &crate::tui::state::VisibleNode, is_highlighted: bool, is_expanded: bool) -> Line<'static> {
        // Use pre-computed indentation to avoid per-frame allocations
        let indent = Self::INDENTS.get(node.level as usize).unwrap_or(&"");

        // Build icon based on node type and expansion state
        let icon = match node.node_type {
            NodeType::Folder => {
                if is_expanded {
                    "[-]"
                } else {
                    "[+]"
                }
            }
            NodeType::Password => " • ",
        };

        // Determine style based on highlight state
        let style = if is_highlighted {
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Build the line efficiently
        let mut spans = Vec::with_capacity(4);
        spans.push(Span::styled(*indent, style));
        spans.push(Span::styled(icon, style));
        spans.push(Span::styled(" ", style));
        spans.push(Span::styled(node.label.clone(), style));

        // Add count only if needed (avoids empty String allocation)
        if node.child_count > 0 && matches!(node.node_type, NodeType::Folder) {
            spans.push(Span::styled(format!(" ({})", node.child_count), style));
        }

        Line::from(spans)
    }

    /// Handle key event with state mutation
    ///
    /// Navigation:
    /// - j/Down: Move to next node
    /// - k/Up: Move to previous node
    /// - g: Move to first node
    /// - G: Move to last node
    ///
    /// Expand/Collapse:
    /// - l/Right: Expand folder
    /// - h/Left: Collapse folder
    /// - Space/Enter: Toggle expand or select
    ///
    /// Selection:
    /// - Enter: Select password and update AppState
    pub fn handle_key_with_state(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
    ) -> HandleResult {
        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        // Pre-fetch current node once for operations that need it
        let current_node = state.tree.current_node();

        match key.code {
            // Navigation: j/down - move down
            KeyCode::Char('j') | KeyCode::Down => {
                self.pending_g = false;
                state.tree.move_down();
                HandleResult::Consumed
            }
            // Navigation: k/up - move up
            KeyCode::Char('k') | KeyCode::Up => {
                self.pending_g = false;
                state.tree.move_up();
                HandleResult::Consumed
            }
            // Navigation: g - move to top (or first of gg sequence)
            KeyCode::Char('g') => {
                let now = Instant::now();

                // Check if this is second 'g' in gg sequence
                if self.pending_g {
                    if let Some(last_time) = self.last_g_time {
                        let elapsed = now.duration_since(last_time).as_millis();
                        if elapsed < u128::from(GG_TIMEOUT_MS) {
                            // Second 'g' within timeout - execute gg jump to top
                            self.pending_g = false;
                            self.last_g_time = None;
                            state.tree.move_to_top();
                        } else {
                            // Timeout expired, treat as new first 'g'
                            self.pending_g = true;
                            self.last_g_time = Some(now);
                        }
                    } else {
                        // No pending state, treat as first 'g'
                        self.pending_g = true;
                        self.last_g_time = Some(now);
                    }
                } else {
                    // No previous key time, start new sequence
                    self.pending_g = true;
                    self.last_g_time = Some(now);
                }
                HandleResult::Consumed
            }
            // Navigation: G (Shift+g) - move to bottom
            KeyCode::Char('G') => {
                self.pending_g = false;
                state.tree.move_to_bottom();
                HandleResult::Consumed
            }
            // Expand: l/right - expand current folder
            KeyCode::Char('l') | KeyCode::Right => {
                self.pending_g = false;
                if let Some(node) = current_node {
                    if let TreeNodeId::Group(id) = node.id {
                        if !state.tree.is_expanded(&id) {
                            state.tree.toggle_expand(id);
                            state.apply_filter();
                        }
                    }
                }
                HandleResult::Consumed
            }
            // Collapse: h/left - collapse current folder
            KeyCode::Char('h') | KeyCode::Left => {
                self.pending_g = false;
                if let Some(node) = current_node {
                    if let TreeNodeId::Group(id) = node.id {
                        if state.tree.is_expanded(&id) {
                            state.tree.toggle_expand(id);
                            state.apply_filter();
                        }
                    }
                }
                HandleResult::Consumed
            }
            // Toggle expand or select
            KeyCode::Char(' ') => {
                self.pending_g = false;
                if let Some(node) = current_node {
                    match node.id {
                        TreeNodeId::Group(id) => {
                            state.tree.toggle_expand(id);
                            state.apply_filter();
                        }
                        TreeNodeId::Password(id) => {
                            state.select_password(id);
                        }
                    }
                }
                HandleResult::Consumed
            }
            // Select: Enter - select password or toggle folder
            KeyCode::Enter => {
                self.pending_g = false;
                if let Some(node) = current_node {
                    match node.id {
                        TreeNodeId::Group(id) => {
                            state.tree.toggle_expand(id);
                            state.apply_filter();
                        }
                        TreeNodeId::Password(id) => {
                            state.select_password(id);
                        }
                    }
                }
                HandleResult::Consumed
            }
            _ => {
                // Any other key clears pending_g
                self.pending_g = false;
                HandleResult::Ignored
            }
        }
    }

    /// Get the currently selected password ID (if any)
    pub fn get_selected_password(&self, state: &TreeState) -> Option<Uuid> {
        state.current_node().and_then(|node| {
            if let TreeNodeId::Password(id) = node.id {
                Some(id)
            } else {
                None
            }
        })
    }
}

impl Default for TreePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for TreePanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Simplified render without state - just show placeholder
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" [1] Groups ");
        block.render(area, buf);
    }
}

impl Interactive for TreePanel {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        // Without state, we can only acknowledge key presses
        match key.code {
            KeyCode::Char('j') | KeyCode::Char('k') | KeyCode::Up | KeyCode::Down => {
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for TreePanel {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focused = true;
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focused = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::state::VisibleNode;

    fn create_visible_node(id: TreeNodeId, level: u8, node_type: NodeType, label: &str) -> VisibleNode {
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
            KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.tree.highlighted_index, 1);

        // Press 'k' to move up
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty()),
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
            KeyEvent::new(KeyCode::Char('g'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        // Single 'g' should not move, index stays at same
        assert_eq!(state.tree.highlighted_index, 1);
        assert!(panel.pending_g);

        // Press 'g' again quickly - should move to top (gg sequence)
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('g'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.tree.highlighted_index, 0);  // Now at top
        assert!(!panel.pending_g);  // pending_g cleared

        // Move to middle for next test
        state.tree.highlighted_index = 1;

        // Press 'G' (Shift+g) to move to bottom
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('G'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.tree.highlighted_index, 2);
    }

    #[test]
    fn test_expand_collapse_folder() {
        let mut panel = TreePanel::new();
        let mut state = AppState::new();

        // Initialize visible_nodes from MockVault (contains real group IDs)
        state.apply_filter();

        // Get the first group ID from visible nodes
        let group_id = match state.tree.current_node() {
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
        assert!(!state.tree.is_expanded(&group_id));

        // Press 'l' to expand
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('l'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert!(state.tree.is_expanded(&group_id));

        // Press 'h' to collapse
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('h'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert!(!state.tree.is_expanded(&group_id));
    }

    #[test]
    fn test_select_password() {
        let mut panel = TreePanel::new();
        let mut state = AppState::new();

        let password_id = Uuid::new_v4();
        state.tree.set_visible_nodes(vec![
            create_visible_node(TreeNodeId::Group(Uuid::new_v4()), 0, NodeType::Folder, "Root"),
            create_visible_node(TreeNodeId::Password(password_id), 1, NodeType::Password, "Entry"),
        ]);
        state.tree.highlighted_index = 1;

        // Initially no selection
        assert!(state.selection.selected_password.is_none());

        // Press Enter to select
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty()),
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
        state.tree.set_visible_nodes(vec![
            create_visible_node(TreeNodeId::Password(password_id), 0, NodeType::Password, "Entry"),
        ]);

        let selected = panel.get_selected_password(&state.tree);
        assert_eq!(selected, Some(password_id));
    }

    #[test]
    fn test_get_selected_password_when_folder() {
        let panel = TreePanel::new();
        let mut state = AppState::new();

        let group_id = Uuid::new_v4();
        state.tree.set_visible_nodes(vec![
            create_visible_node(TreeNodeId::Group(group_id), 0, NodeType::Folder, "Folder"),
        ]);

        let selected = panel.get_selected_password(&state.tree);
        assert!(selected.is_none());
    }
}
