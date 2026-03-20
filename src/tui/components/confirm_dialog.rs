//! Confirm Dialog Component
//!
//! A modal dialog that asks for user confirmation before performing an action.
//! Supports customizable title, message, and action buttons.

use crate::tui::error::TuiResult;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Confirm dialog action type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Delete password (move to trash)
    DeletePassword {
        password_id: String,
        password_name: String,
    },
    /// Permanently delete
    PermanentDelete(String),
    /// Empty trash
    EmptyTrash,
    /// Generic confirmation
    Generic,
}

/// Confirm dialog component
pub struct ConfirmDialog {
    /// Component ID
    id: ComponentId,
    /// Dialog title
    title: String,
    /// Main message
    message: String,
    /// Additional details (shown in gray)
    details: Option<String>,
    /// Confirm button label
    confirm_label: String,
    /// Cancel button label
    cancel_label: String,
    /// Currently focused button (true = confirm, false = cancel)
    focused_on_confirm: bool,
    /// The action to perform on confirmation
    action: ConfirmAction,
    /// Whether the dialog is visible
    visible: bool,
}

impl ConfirmDialog {
    /// Create a new confirm dialog
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            title: "Confirm".to_string(),
            message: String::new(),
            details: None,
            confirm_label: "Confirm".to_string(),
            cancel_label: "Cancel".to_string(),
            focused_on_confirm: false,
            action: ConfirmAction::Generic,
            visible: false,
        }
    }

    /// Create a delete confirmation dialog for a password
    pub fn delete_confirmation(password_name: &str, password_id: &str) -> Self {
        Self {
            id: ComponentId::new(0),
            title: "⚠️  Confirm Delete".to_string(),
            message: format!("Move \"{}\" to trash?", password_name),
            details: Some("This password will be permanently deleted after 30 days.".to_string()),
            confirm_label: "Delete".to_string(),
            cancel_label: "Cancel".to_string(),
            focused_on_confirm: false,
            action: ConfirmAction::DeletePassword {
                password_id: password_id.to_string(),
                password_name: password_name.to_string(),
            },
            visible: true,
        }
    }

    /// Create a permanent delete confirmation dialog
    pub fn permanent_delete_confirmation(password_name: &str, password_id: &str) -> Self {
        Self {
            id: ComponentId::new(0),
            title: "⚠️  Permanent Delete".to_string(),
            message: format!("Permanently delete \"{}\"?", password_name),
            details: Some("This action cannot be undone!".to_string()),
            confirm_label: "Delete Forever".to_string(),
            cancel_label: "Cancel".to_string(),
            focused_on_confirm: false,
            action: ConfirmAction::PermanentDelete(password_id.to_string()),
            visible: true,
        }
    }

    /// Create an empty trash confirmation dialog
    pub fn empty_trash_confirmation(count: usize) -> Self {
        Self {
            id: ComponentId::new(0),
            title: "⚠️  Empty Trash".to_string(),
            message: format!("Permanently delete {} item(s)?", count),
            details: Some("This action cannot be undone!".to_string()),
            confirm_label: "Empty Trash".to_string(),
            cancel_label: "Cancel".to_string(),
            focused_on_confirm: false,
            action: ConfirmAction::EmptyTrash,
            visible: true,
        }
    }

    /// Set component ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// Check if the dialog is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Show the dialog
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Get the action to perform
    pub fn action(&self) -> &ConfirmAction {
        &self.action
    }

    /// Check if confirm button is focused
    pub fn is_confirm_focused(&self) -> bool {
        self.focused_on_confirm
    }

    /// Toggle focus between buttons
    pub fn toggle_focus(&mut self) {
        self.focused_on_confirm = !self.focused_on_confirm;
    }

    /// Check if user confirmed
    pub fn is_confirmed(&self) -> bool {
        self.focused_on_confirm
    }
}

impl Default for ConfirmDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ConfirmDialog {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        // Calculate dialog size (centered)
        let dialog_width = 50.min(area.width);
        let dialog_height = 8.min(area.height);

        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

        // Clear the area behind the dialog
        Clear.render(dialog_area, buf);

        // Draw dialog border
        let block = Block::default()
            .title(Span::styled(
                &self.title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Layout for content and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message area
                Constraint::Length(1), // Details (if any)
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Buttons
            ])
            .split(inner);

        // Render message
        let message = Paragraph::new(Line::from(Span::styled(
            &self.message,
            Style::default().fg(Color::White),
        )))
        .alignment(Alignment::Center);
        message.render(chunks[0], buf);

        // Render details if present
        if let Some(ref details) = self.details {
            let details_para = Paragraph::new(Line::from(Span::styled(
                details,
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center);
            details_para.render(chunks[1], buf);
        }

        // Render buttons
        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[3]);

        // Cancel button (left)
        let cancel_style = if self.focused_on_confirm {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::REVERSED)
        };
        let cancel_text = format!("[ {} ]", self.cancel_label);
        let cancel_btn = Paragraph::new(Line::from(Span::styled(cancel_text, cancel_style)))
            .alignment(Alignment::Center);
        cancel_btn.render(button_chunks[0], buf);

        // Confirm button (right)
        let confirm_style = if self.focused_on_confirm {
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let confirm_text = format!("[ {} ]", self.confirm_label);
        let confirm_btn = Paragraph::new(Line::from(Span::styled(confirm_text, confirm_style)))
            .alignment(Alignment::Center);
        confirm_btn.render(button_chunks[1], buf);
    }
}

impl Interactive for ConfirmDialog {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            // Tab/Left/Right: Toggle between buttons
            KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                self.toggle_focus();
                HandleResult::Consumed
            }
            // Enter: Confirm action (based on current focus)
            KeyCode::Enter => {
                if self.focused_on_confirm {
                    // User confirmed - return the action to execute
                    HandleResult::Action(crate::tui::traits::Action::ConfirmDialog(
                        self.action.clone(),
                    ))
                } else {
                    // User cancelled - close dialog without action
                    HandleResult::Action(crate::tui::traits::Action::CloseScreen)
                }
            }
            // Escape: Cancel
            KeyCode::Esc => HandleResult::Action(crate::tui::traits::Action::CloseScreen),
            // y/Y: Confirm
            KeyCode::Char('y') | KeyCode::Char('Y') => HandleResult::Action(
                crate::tui::traits::Action::ConfirmDialog(self.action.clone()),
            ),
            // n/N: Cancel
            KeyCode::Char('n') | KeyCode::Char('N') => {
                HandleResult::Action(crate::tui::traits::Action::CloseScreen)
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for ConfirmDialog {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        self.visible
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_dialog_creation() {
        let dialog = ConfirmDialog::new();
        assert!(!dialog.is_visible());
        assert!(!dialog.is_confirm_focused());
    }

    #[test]
    fn test_delete_confirmation() {
        let dialog = ConfirmDialog::delete_confirmation("My Password", "id-123");
        assert!(dialog.is_visible());
        assert_eq!(dialog.title, "⚠️  Confirm Delete");
        assert!(
            matches!(dialog.action, ConfirmAction::DeletePassword { password_id, .. } if password_id == "id-123")
        );
    }

    #[test]
    fn test_permanent_delete_confirmation() {
        let dialog = ConfirmDialog::permanent_delete_confirmation("My Password", "id-456");
        assert!(dialog.is_visible());
        assert!(matches!(dialog.action, ConfirmAction::PermanentDelete(id) if id == "id-456"));
    }

    #[test]
    fn test_empty_trash_confirmation() {
        let dialog = ConfirmDialog::empty_trash_confirmation(5);
        assert!(dialog.is_visible());
        assert!(matches!(dialog.action, ConfirmAction::EmptyTrash));
    }

    #[test]
    fn test_toggle_focus() {
        let mut dialog = ConfirmDialog::new();
        assert!(!dialog.is_confirm_focused());

        dialog.toggle_focus();
        assert!(dialog.is_confirm_focused());

        dialog.toggle_focus();
        assert!(!dialog.is_confirm_focused());
    }

    #[test]
    fn test_show_hide() {
        let mut dialog = ConfirmDialog::new();
        assert!(!dialog.is_visible());

        dialog.show();
        assert!(dialog.is_visible());

        dialog.hide();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_handle_key_tab() {
        let mut dialog = ConfirmDialog::new();
        dialog.show();

        let key = KeyEvent::new(KeyCode::Tab, crossterm::event::KeyModifiers::empty());
        let result = dialog.handle_key(key);
        assert!(matches!(result, HandleResult::Consumed));
        assert!(dialog.is_confirm_focused());
    }

    #[test]
    fn test_handle_key_escape() {
        let mut dialog = ConfirmDialog::new();
        dialog.show();

        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());
        let result = dialog.handle_key(key);
        assert!(matches!(
            result,
            HandleResult::Action(crate::tui::traits::Action::CloseScreen)
        ));
    }

    #[test]
    fn test_handle_key_enter() {
        let mut dialog = ConfirmDialog::new();
        dialog.show();
        dialog.toggle_focus(); // Focus on confirm

        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty());
        let result = dialog.handle_key(key);
        // When confirmed (focused on confirm button), returns the action
        assert!(matches!(
            result,
            HandleResult::Action(crate::tui::traits::Action::ConfirmDialog(_))
        ));
    }

    #[test]
    fn test_handle_key_y_yes() {
        let mut dialog = ConfirmDialog::new();
        dialog.show();

        let key = KeyEvent::new(KeyCode::Char('y'), crossterm::event::KeyModifiers::empty());
        let result = dialog.handle_key(key);
        // Y key confirms, returns the action
        assert!(matches!(
            result,
            HandleResult::Action(crate::tui::traits::Action::ConfirmDialog(_))
        ));
    }

    #[test]
    fn test_handle_key_n_no() {
        let mut dialog = ConfirmDialog::new();
        dialog.show();

        let key = KeyEvent::new(KeyCode::Char('n'), crossterm::event::KeyModifiers::empty());
        let result = dialog.handle_key(key);
        // N key cancels, closes screen
        assert!(matches!(
            result,
            HandleResult::Action(crate::tui::traits::Action::CloseScreen)
        ));
    }
}
