//! Key event handlers for TreePanel
//!
//! Contains keyboard handling logic for tree navigation.

use super::{TreeEditMode, TreePanel, GG_TIMEOUT_MS};
use crate::tui::state::{AppState, TreeNodeId};
use crate::tui::traits::{Action, HandleResult, NotificationLevel};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::time::Instant;

/// Handle key event with state mutation
pub fn handle_key_with_state(
    panel: &mut TreePanel,
    key: KeyEvent,
    state: &mut AppState,
) -> HandleResult {
    if key.kind == KeyEventKind::Release {
        return HandleResult::Ignored;
    }

    if panel.edit_mode != TreeEditMode::None {
        return handle_edit_mode_key(panel, key, state);
    }

    let current_node = state.tree.current_node();

    match key.code {
        // Navigation: j/down - move down
        KeyCode::Char('j') | KeyCode::Down => {
            panel.pending_g = false;
            state.tree.move_down();
            HandleResult::Consumed
        }
        // Navigation: k/up - move up
        KeyCode::Char('k') | KeyCode::Up => {
            panel.pending_g = false;
            state.tree.move_up();
            HandleResult::Consumed
        }
        // Navigation: g - move to top (or first of gg sequence)
        KeyCode::Char('g') => handle_g_key(panel, state),
        // Navigation: G (Shift+g) - move to bottom
        KeyCode::Char('G') => {
            panel.pending_g = false;
            state.tree.move_to_bottom();
            HandleResult::Consumed
        }
        // Expand: l/right - expand current folder
        KeyCode::Char('l') | KeyCode::Right => {
            panel.pending_g = false;
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
            panel.pending_g = false;
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
            panel.pending_g = false;
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
            panel.pending_g = false;
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
        // Group editing: a - create new group
        KeyCode::Char('a') => {
            panel.pending_g = false;
            panel.edit_mode = TreeEditMode::CreatingGroup;
            panel.edit_buffer.clear();
            HandleResult::Consumed
        }
        // Group editing: r - rename current group
        KeyCode::Char('r') => {
            panel.pending_g = false;
            if let Some(node) = state.tree.current_node() {
                if let TreeNodeId::Group(gid) = node.id {
                    if gid != uuid::Uuid::nil() {
                        let name = state
                            .groups
                            .iter()
                            .find(|g| uuid::Uuid::parse_str(&g.id).ok() == Some(gid))
                            .map(|g| g.name.clone())
                            .unwrap_or_default();
                        panel.edit_mode =
                            TreeEditMode::RenamingGroup { group_id: gid.to_string() };
                        panel.edit_buffer = name;
                        return HandleResult::Consumed;
                    }
                }
            }
            HandleResult::Ignored
        }
        // Group editing: D (shift+d) - delete current group
        KeyCode::Char('D') => {
            panel.pending_g = false;
            if let Some(node) = state.tree.current_node() {
                if let TreeNodeId::Group(gid) = node.id {
                    if gid != uuid::Uuid::nil() {
                        let name = state
                            .groups
                            .iter()
                            .find(|g| uuid::Uuid::parse_str(&g.id).ok() == Some(gid))
                            .map(|g| g.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        return HandleResult::Action(Action::OpenScreen(
                            crate::tui::traits::ScreenType::ConfirmDialog(
                                crate::tui::components::ConfirmAction::DeleteGroup {
                                    group_id: gid.to_string(),
                                    group_name: name,
                                },
                            ),
                        ));
                    }
                }
            }
            HandleResult::Ignored
        }
        // Group editing: m - move current password to a group
        KeyCode::Char('m') => {
            panel.pending_g = false;
            if let Some(node) = state.tree.current_node() {
                if let TreeNodeId::Password(_pid) = node.id {
                    return HandleResult::Action(Action::ShowToast(
                        "__show_group_picker".to_string(),
                    ));
                }
            }
            HandleResult::Ignored
        }
        _ => {
            panel.pending_g = false;
            HandleResult::Ignored
        }
    }
}

/// Handle key events when in edit mode (creating/renaming group)
fn handle_edit_mode_key(
    panel: &mut TreePanel,
    key: KeyEvent,
    state: &mut AppState,
) -> HandleResult {
    match key.code {
        KeyCode::Esc => {
            panel.edit_mode = TreeEditMode::None;
            panel.edit_buffer.clear();
            HandleResult::Consumed
        }
        KeyCode::Enter => {
            let name = panel.edit_buffer.trim().to_string();
            if name.is_empty() {
                state.add_notification(
                    "Group name cannot be empty",
                    NotificationLevel::Warning,
                );
                return HandleResult::Consumed;
            }
            match &panel.edit_mode {
                TreeEditMode::CreatingGroup => {
                    panel.edit_mode = TreeEditMode::None;
                    let buffer = panel.edit_buffer.clone();
                    panel.edit_buffer.clear();
                    HandleResult::Action(Action::ShowToast(format!("__create_group:{}", buffer)))
                }
                TreeEditMode::RenamingGroup { group_id } => {
                    let gid = group_id.clone();
                    panel.edit_mode = TreeEditMode::None;
                    let buffer = panel.edit_buffer.clone();
                    panel.edit_buffer.clear();
                    HandleResult::Action(Action::ShowToast(format!(
                        "__rename_group:{}:{}",
                        gid, buffer
                    )))
                }
                _ => HandleResult::Consumed,
            }
        }
        KeyCode::Char(c) if panel.edit_buffer.len() < 32 => {
            panel.edit_buffer.push(c);
            HandleResult::NeedsRender
        }
        KeyCode::Backspace => {
            panel.edit_buffer.pop();
            HandleResult::NeedsRender
        }
        _ => HandleResult::Consumed,
    }
}

/// Handle 'g' key for gg double-key sequence
fn handle_g_key(panel: &mut TreePanel, state: &mut AppState) -> HandleResult {
    let now = Instant::now();

    if panel.pending_g {
        if let Some(last_time) = panel.last_g_time {
            let elapsed = now.duration_since(last_time).as_millis();
            if elapsed < GG_TIMEOUT_MS {
                // Second 'g' within timeout - execute gg jump to top
                panel.pending_g = false;
                panel.last_g_time = None;
                state.tree.move_to_top();
            } else {
                // Timeout expired, treat as new first 'g'
                panel.pending_g = true;
                panel.last_g_time = Some(now);
            }
        } else {
            // No pending state, treat as first 'g'
            panel.pending_g = true;
            panel.last_g_time = Some(now);
        }
    } else {
        // Start new sequence
        panel.pending_g = true;
        panel.last_g_time = Some(now);
    }
    HandleResult::Consumed
}
