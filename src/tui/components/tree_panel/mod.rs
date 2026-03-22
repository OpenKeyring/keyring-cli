//! Tree Panel Component
//!
//! Displays a tree view of password groups and entries.
//! Supports Vim-style navigation (j/k/g/G/gg) and expand/collapse (h/l).

use crate::tui::error::TuiResult;
use crate::tui::state::{AppState, TreeNodeId, TreeState};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    widgets::{Block, Borders},
    Frame,
};
use std::time::Instant;
use uuid::Uuid;

mod handlers;
mod render;
#[cfg(test)]
mod tests;

/// Maximum time between gg key presses (in milliseconds)
pub(super) const GG_TIMEOUT_MS: u128 = 500;

/// Tree panel editing mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEditMode {
    /// Normal mode
    None,
    /// Creating a new group
    CreatingGroup,
    /// Renaming a group
    RenamingGroup { group_id: String },
}

impl Default for TreeEditMode {
    fn default() -> Self {
        Self::None
    }
}

/// Tree panel component
///
/// Displays a hierarchical tree of groups and password entries.
/// Uses TreeState for navigation state, tracks gg double-key internally.
pub struct TreePanel {
    /// Component ID
    id: ComponentId,
    /// Whether the panel has focus
    pub(super) focused: bool,
    /// Track pending 'g' key for gg double-key sequence
    pub(super) pending_g: bool,
    /// Time of last 'g' key press
    pub(super) last_g_time: Option<Instant>,
    /// Current editing mode
    pub(super) edit_mode: TreeEditMode,
    /// Text buffer for inline editing
    pub(super) edit_buffer: String,
}

impl TreePanel {
    /// Create a new tree panel
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            focused: false,
            pending_g: false,
            last_g_time: None,
            edit_mode: TreeEditMode::None,
            edit_buffer: String::new(),
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
    pub fn render_frame(&self, frame: &mut Frame, area: Rect, state: &TreeState) {
        self.render_frame_with_context(frame, area, state, false)
    }

    /// Render to frame with filter context for better empty state messages
    pub fn render_frame_with_context(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &TreeState,
        has_active_filters: bool,
    ) {
        render::render_frame_with_context(
            frame,
            area,
            state,
            &render::RenderContext {
                focused: self.focused,
            },
            has_active_filters,
        );
    }

    /// Handle key event with state mutation
    pub fn handle_key_with_state(
        &mut self,
        key: crossterm::event::KeyEvent,
        state: &mut AppState,
    ) -> HandleResult {
        handlers::handle_key_with_state(self, key, state)
    }

    /// Get the currently selected password ID (if any)
    pub fn get_selected_password(&self, state: &crate::tui::state::TreeState) -> Option<Uuid> {
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
        let block = Block::default().borders(Borders::ALL).title(" [1] Groups ");
        block.render(area, buf);
    }
}

impl Interactive for TreePanel {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> HandleResult {
        use crossterm::event::{KeyCode, KeyEventKind};

        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

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
