//! Detail Panel Component
//!
//! Displays detailed information about the selected password entry.
//! Supports password visibility toggle and copy-to-clipboard actions.

mod handlers;
mod render;
#[cfg(test)]
mod tests;

use crate::tui::error::TuiResult;
use crate::tui::models::password::PasswordRecord;
use crate::tui::state::{AppState, DetailMode};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
    Frame,
};

/// Detail panel component
///
/// Displays full details of a selected password entry.
/// Supports keyboard actions for copying and toggling password visibility.
pub struct DetailPanel {
    /// Component ID
    id: ComponentId,
    /// Whether the panel has focus
    pub(super) focused: bool,
    /// Whether password is visible (shown as plain text)
    pub(super) password_visible: bool,
}

impl DetailPanel {
    /// Create a new detail panel
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            focused: false,
            password_visible: false,
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

    /// Toggle password visibility
    pub fn toggle_password_visibility(&mut self) {
        self.password_visible = !self.password_visible;
    }

    /// Check if password is visible
    pub fn is_password_visible(&self) -> bool {
        self.password_visible
    }

    /// Render to frame (preferred method)
    pub fn render_frame(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if area.height < 5 {
            return;
        }

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
            .title(" [3] Details ");

        let inner_area = block.inner(area);
        block.render(area, frame.buffer_mut());

        match &state.detail_mode {
            DetailMode::ProjectInfo => {
                render::render_project_info(frame, inner_area);
            }
            DetailMode::PasswordDetail(id) => {
                if let Some(password) = state.get_password(*id) {
                    render::render_password(frame, inner_area, password, self.password_visible);
                } else {
                    render::render_project_info(frame, inner_area);
                }
            }
        }
    }

    /// Handle key event with state mutation
    pub fn handle_key_with_state(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        _password: Option<&PasswordRecord>,
    ) -> HandleResult {
        handlers::handle_key_with_state(self, key, state)
    }
}

impl Default for DetailPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for DetailPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" [3] Details ");
        block.render(area, buf);
    }
}

impl Interactive for DetailPanel {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char(' ') => {
                self.toggle_password_visibility();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for DetailPanel {
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
        self.password_visible = false;
        Ok(())
    }
}
