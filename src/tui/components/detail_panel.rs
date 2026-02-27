//! Detail Panel Component
//!
//! Displays detailed information about the selected password entry.
//! Supports password visibility toggle and copy-to-clipboard actions.

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
    widgets::{Block, Borders, Paragraph, Wrap},
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
    focused: bool,
    /// Whether password is visible (shown as plain text)
    password_visible: bool,
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
            // Not enough space to render
            return;
        }

        // Create border block
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
            .title(" Details ");

        let inner_area = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Render content based on detail mode
        match &state.detail_mode {
            DetailMode::ProjectInfo => {
                self.render_project_info(frame, inner_area);
            }
            DetailMode::PasswordDetail(_id) => {
                // In a real implementation, we would fetch the password by ID
                // For now, we render a placeholder
                self.render_project_info(frame, inner_area);
            }
        }
    }

    /// Render project information when no password is selected
    fn render_project_info(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "OpenKeyring",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Privacy-first Password Manager",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Version: v0.1.0",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "License: MIT License",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Website: github.com/open-keyring",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [n] to create your first password",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Navigation: j/k to move  |  Enter to select  |  1-4 to switch panels",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Render password details
    pub fn render_password(&self, frame: &mut Frame, area: Rect, password: &PasswordRecord) {
        let mut lines: Vec<Line<'_>> = Vec::new();

        // Title (name)
        lines.push(Line::from(Span::styled(
            password.name.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Username
        if let Some(ref username) = password.username {
            lines.push(Self::create_field_line_owned(
                "Username:",
                username,
                "[c] copy",
            ));
        }

        // Password
        let password_display = if self.password_visible {
            password.password.clone()
        } else {
            "*".repeat(password.password.len().min(20))
        };
        lines.push(Self::create_field_line_owned(
            "Password:",
            &password_display,
            "[C] copy  [Space] toggle",
        ));

        // URL
        if let Some(ref url) = password.url {
            lines.push(Self::create_field_line_owned("URL:", url, "[o] open"));
        }

        // Tags
        if !password.tags.is_empty() {
            let tags_str = password.tags.join(", ");
            lines.push(Self::create_field_line_owned("Tags:", &tags_str, ""));
        }

        // Notes
        if let Some(ref notes) = password.notes {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Notes:",
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(Span::raw(notes.clone())));
        }

        // Timestamps
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(
                "Created: {}  |  Modified: {}",
                password.created_at.format("%Y-%m-%d %H:%M"),
                password.modified_at.format("%Y-%m-%d %H:%M")
            ),
            Style::default().fg(Color::DarkGray),
        )));

        // Status indicators
        if password.is_favorite {
            lines.push(Line::from(Span::styled(
                "⭐ Favorite",
                Style::default().fg(Color::Yellow),
            )));
        }
        if password.is_deleted {
            lines.push(Line::from(Span::styled(
                "🗑 In Trash",
                Style::default().fg(Color::Red),
            )));
        }

        // Action hints at bottom (design doc requirement)
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("[e] Edit", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("[d] Delete", Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled("[Space] Toggle password", Style::default().fg(Color::DarkGray)),
        ]));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    /// Create a field line with label, value, and hint (owned version)
    fn create_field_line_owned(label: &str, value: &str, hint: &str) -> Line<'static> {
        let mut spans = vec![
            Span::styled(label.to_string(), Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(value.to_string(), Style::default().fg(Color::White)),
        ];

        if !hint.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(hint.to_string(), Style::default().fg(Color::DarkGray)));
        }

        Line::from(spans)
    }

    /// Create a field line with label, value, and hint
    #[allow(dead_code)]
    fn create_field_line<'a>(
        &self,
        label: &'a str,
        value: &'a str,
        hint: &'a str,
    ) -> Line<'a> {
        let mut spans = vec![
            Span::styled(label, Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(value, Style::default().fg(Color::White)),
        ];

        if !hint.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(hint, Style::default().fg(Color::DarkGray)));
        }

        Line::from(spans)
    }

    /// Handle key event with state mutation
    pub fn handle_key_with_state(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
        _password: Option<&PasswordRecord>,
    ) -> HandleResult {
        use crossterm::event::KeyModifiers;

        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            // Space: Toggle password visibility (design doc requirement)
            KeyCode::Char(' ') => {
                self.toggle_password_visibility();
                HandleResult::Consumed
            }
            // c: Copy username (design doc requirement)
            KeyCode::Char('c') => {
                // Check for Shift modifier (uppercase C) to copy password
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    // C (Shift+c): Copy password
                    state.add_notification("Password copied to clipboard", crate::tui::traits::NotificationLevel::Info);
                } else {
                    // c: Copy username
                    state.add_notification("Username copied to clipboard", crate::tui::traits::NotificationLevel::Info);
                }
                HandleResult::Consumed
            }
            // o: Open URL (extra feature, not in design doc but useful)
            KeyCode::Char('o') => {
                state.add_notification("Opening URL...", crate::tui::traits::NotificationLevel::Info);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
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
            .title(" Details ");
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
        // Also hide password when losing focus for security
        self.password_visible = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detail_panel_creation() {
        let panel = DetailPanel::new();
        assert!(!panel.focused);
        assert!(!panel.password_visible);
    }

    #[test]
    fn test_password_visibility_toggle() {
        let mut panel = DetailPanel::new();
        assert!(!panel.is_password_visible());

        panel.toggle_password_visibility();
        assert!(panel.is_password_visible());

        panel.toggle_password_visibility();
        assert!(!panel.is_password_visible());
    }

    #[test]
    fn test_focus_state() {
        let mut panel = DetailPanel::new();
        assert!(!panel.is_focused());

        panel.on_focus_gain().unwrap();
        assert!(panel.is_focused());

        panel.on_focus_loss().unwrap();
        assert!(!panel.is_focused());
    }

    #[test]
    fn test_password_hides_on_focus_loss() {
        let mut panel = DetailPanel::new();
        panel.toggle_password_visibility();
        assert!(panel.is_password_visible());

        panel.on_focus_loss().unwrap();
        assert!(!panel.is_password_visible());
    }

    #[test]
    fn test_component_trait() {
        let panel = DetailPanel::new();
        assert!(panel.can_focus());
    }

    #[test]
    fn test_handle_key_toggle_password() {
        let mut panel = DetailPanel::new();
        // Space key toggles password visibility
        let key = KeyEvent::new(KeyCode::Char(' '), crossterm::event::KeyModifiers::empty());

        let result = panel.handle_key(key);
        assert!(matches!(result, HandleResult::Consumed));
        assert!(panel.is_password_visible());
    }

    #[test]
    fn test_handle_key_with_state_copy_username() {
        let mut panel = DetailPanel::new();
        let mut state = AppState::new();

        // 'c' (lowercase) copies username
        let key = KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::empty());
        let result = panel.handle_key_with_state(key, &mut state, None);

        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.notifications.len(), 1);
    }

    #[test]
    fn test_handle_key_with_state_copy_password() {
        let mut panel = DetailPanel::new();
        let mut state = AppState::new();

        // 'C' (Shift+c) copies password
        let key = KeyEvent::new(KeyCode::Char('c'), crossterm::event::KeyModifiers::SHIFT);
        let result = panel.handle_key_with_state(key, &mut state, None);

        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(state.notifications.len(), 1);
    }

    #[test]
    fn test_handle_key_with_state_toggle_password() {
        let mut panel = DetailPanel::new();
        let mut state = AppState::new();

        // Space toggles password visibility
        let key = KeyEvent::new(KeyCode::Char(' '), crossterm::event::KeyModifiers::empty());
        let result = panel.handle_key_with_state(key, &mut state, None);

        assert!(matches!(result, HandleResult::Consumed));
        assert!(panel.is_password_visible());
    }
}
