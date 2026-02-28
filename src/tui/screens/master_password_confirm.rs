//! Master Password Confirmation Screen
//!
//! Second step of password setup - user must re-enter password to confirm

use crate::tui::error::TuiResult;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Master password confirmation screen
pub struct MasterPasswordConfirmScreen {
    /// Input buffer
    input: String,
    /// The original password to match against
    original_password: Option<String>,
    /// Error message if passwords don't match
    error: Option<String>,
    /// Component ID
    id: ComponentId,
}

impl MasterPasswordConfirmScreen {
    /// Create new confirmation screen
    pub fn new() -> Self {
        Self {
            input: String::new(),
            original_password: None,
            error: None,
            id: ComponentId::new(3010),
        }
    }

    /// Create with the original password to verify
    pub fn with_original_password(password: String) -> Self {
        Self {
            input: String::new(),
            original_password: Some(password),
            error: None,
            id: ComponentId::new(3010),
        }
    }

    /// Set the original password to verify against
    pub fn set_original_password(&mut self, password: String) {
        self.original_password = Some(password);
    }

    /// Get the confirmed password
    pub fn password(&self) -> &str {
        &self.input
    }

    /// Check if passwords match
    pub fn passwords_match(&self) -> bool {
        match &self.original_password {
            Some(original) => self.input == *original && !self.input.is_empty(),
            None => false,
        }
    }

    /// Set error message
    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Clear input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.error = None;
    }
}

impl Default for MasterPasswordConfirmScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for MasterPasswordConfirmScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("🔐 Confirm Master Password")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        // Password strength indicator
        let mut lines = vec![
            Line::from(""),
            Line::from("Please re-enter your master password to confirm."),
            Line::from(""),
            Line::from(vec![
                Span::raw("Confirm Password: "),
                Span::styled(
                    "*".repeat(self.input.len().min(32)),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("█", Style::default().fg(Color::White)),
            ]),
        ];

        // Show match status if we have input
        if !self.input.is_empty() {
            if let Some(original) = &self.original_password {
                let (icon, color, msg) = if self.input == *original {
                    ("✓", Color::Green, "Passwords match")
                } else if original.starts_with(&self.input) {
                    ("...", Color::Yellow, "")
                } else {
                    ("✗", Color::Red, "Passwords do not match")
                };
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(msg, Style::default().fg(color)),
                ]));
            }
        }

        // Show error if any
        if let Some(err) = &self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("❌ {}", err),
                Style::default().fg(Color::Red),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(
            Line::from("[Enter] Confirm   [Esc] Back   [Ctrl+U] Clear")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        paragraph.render(inner, buf);
    }
}

impl Interactive for MasterPasswordConfirmScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Char(c) => {
                if self.input.len() < 128 {
                    self.input.push(c);
                    self.error = None;
                }
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.error = None;
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.passwords_match() {
                    HandleResult::Consumed
                } else {
                    self.error = Some("Passwords do not match. Please try again.".to_string());
                    HandleResult::NeedsRender
                }
            }
            // Ctrl+U to clear
            KeyCode::Char('u') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.clear_input();
                HandleResult::NeedsRender
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for MasterPasswordConfirmScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_screen() {
        let screen = MasterPasswordConfirmScreen::new();
        assert!(screen.input.is_empty());
        assert!(screen.original_password.is_none());
        assert!(!screen.passwords_match());
    }

    #[test]
    fn test_with_original_password() {
        let screen = MasterPasswordConfirmScreen::with_original_password("test123".to_string());
        assert_eq!(screen.original_password, Some("test123".to_string()));
    }

    #[test]
    fn test_passwords_match() {
        let mut screen = MasterPasswordConfirmScreen::with_original_password("test123".to_string());

        screen.input = "test".to_string();
        assert!(!screen.passwords_match());

        screen.input = "test123".to_string();
        assert!(screen.passwords_match());
    }

    #[test]
    fn test_handle_char_input() {
        let mut screen = MasterPasswordConfirmScreen::new();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Char('a')));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert_eq!(screen.input, "a");
    }

    #[test]
    fn test_handle_backspace() {
        let mut screen = MasterPasswordConfirmScreen::new();
        screen.input = "abc".to_string();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Backspace));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert_eq!(screen.input, "ab");
    }

    #[test]
    fn test_enter_with_matching_passwords() {
        let mut screen = MasterPasswordConfirmScreen::with_original_password("test".to_string());
        screen.input = "test".to_string();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }

    #[test]
    fn test_enter_with_non_matching_passwords() {
        let mut screen = MasterPasswordConfirmScreen::with_original_password("test".to_string());
        screen.input = "wrong".to_string();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert!(screen.error.is_some());
    }

    #[test]
    fn test_clear_input() {
        let mut screen = MasterPasswordConfirmScreen::new();
        screen.input = "test".to_string();
        screen.error = Some("error".to_string());
        screen.clear_input();
        assert!(screen.input.is_empty());
        assert!(screen.error.is_none());
    }

    #[test]
    fn test_set_error() {
        let mut screen = MasterPasswordConfirmScreen::new();
        screen.set_error("Test error".to_string());
        assert_eq!(screen.error, Some("Test error".to_string()));
    }
}
