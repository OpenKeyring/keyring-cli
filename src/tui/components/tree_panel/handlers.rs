//! Key event handlers for TreePanel
//!
//! Contains keyboard handling logic for tree navigation.

use super::{TreePanel, GG_TIMEOUT_MS};
use crate::tui::state::{AppState, TreeNodeId};
use crate::tui::traits::HandleResult;
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
        _ => {
            panel.pending_g = false;
            HandleResult::Ignored
        }
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
