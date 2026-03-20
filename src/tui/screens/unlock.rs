//! Unlock Screen
//!
//! Allows users to unlock the vault by entering their master password.
//! Shown when keystore exists but TUI needs to decrypt data.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Unlock state for tracking unlock attempts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlockState {
    /// Waiting for user input
    Input,
    /// Currently attempting to unlock
    Unlocking,
    /// Unlock failed (wrong password)
    Failed,
}

/// Unlock screen for entering master password
#[derive(Debug, Clone)]
pub struct UnlockScreen {
    /// Password input (masked)
    password_input: String,
    /// Current unlock state
    state: UnlockState,
    /// Error message (if any)
    error_message: Option<String>,
    /// Number of failed attempts
    failed_attempts: u8,
}

impl UnlockScreen {
    /// Create a new unlock screen
    pub fn new() -> Self {
        Self {
            password_input: String::new(),
            state: UnlockState::Input,
            error_message: None,
            failed_attempts: 0,
        }
    }

    /// Get current password input
    pub fn password(&self) -> &str {
        &self.password_input
    }

    /// Get current state
    pub fn state(&self) -> UnlockState {
        self.state
    }

    /// Check if ready to attempt unlock
    pub fn can_unlock(&self) -> bool {
        !self.password_input.is_empty() && self.state == UnlockState::Input
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if c.is_control() || self.state == UnlockState::Unlocking {
            return;
        }
        // Reset from Failed state when user starts typing
        if self.state == UnlockState::Failed {
            self.state = UnlockState::Input;
        }
        self.password_input.push(c);
        self.error_message = None;
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.state == UnlockState::Unlocking {
            return;
        }
        self.password_input.pop();
        self.error_message = None;
    }

    /// Clear password input
    pub fn clear(&mut self) {
        self.password_input.clear();
        self.error_message = None;
    }

    /// Set state to unlocking (when attempting to decrypt)
    pub fn set_unlocking(&mut self) {
        self.state = UnlockState::Unlocking;
        self.error_message = None;
    }

    /// Set unlock success (transition to main screen)
    pub fn set_success(&mut self) {
        self.state = UnlockState::Input;
        self.password_input.clear();
        self.failed_attempts = 0;
    }

    /// Set unlock failed with error message
    pub fn set_failed(&mut self, error: &str) {
        self.state = UnlockState::Failed;
        self.error_message = Some(error.to_string());
        self.failed_attempts = self.failed_attempts.saturating_add(1);
        self.password_input.clear();
    }

    /// Get failed attempts count
    pub fn failed_attempts(&self) -> u8 {
        self.failed_attempts
    }

    /// Render the unlock screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(5), // Password input
                    Constraint::Length(3), // Status/Error
                    Constraint::Length(2), // Spacer
                    Constraint::Min(0),    // Info
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title with lock icon
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "🔒 Unlock OpenKeyring",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(title, chunks[0]);

        // Password input field
        let password_display = "•".repeat(self.password_input.len());
        let input_style = match self.state {
            UnlockState::Input => Style::default().fg(Color::Cyan),
            UnlockState::Unlocking => Style::default().fg(Color::Yellow),
            UnlockState::Failed => Style::default().fg(Color::Red),
        };

        let placeholder = match self.state {
            UnlockState::Input if self.password_input.is_empty() => "Enter master password...",
            UnlockState::Unlocking => "Unlocking...",
            UnlockState::Failed => "Try again...",
            _ => "",
        };

        let password_field = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Master Password: ", input_style),
                Span::styled(
                    if password_display.is_empty() {
                        placeholder
                    } else {
                        &password_display
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(input_style)
                .title(" Password "),
        )
        .wrap(Wrap { trim: false });

        frame.render_widget(password_field, chunks[2]);

        // Status/Error message
        let status = match &self.state {
            UnlockState::Unlocking => Paragraph::new(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled("Unlocking vault...", Style::default().fg(Color::Yellow)),
            ])),
            UnlockState::Failed => {
                if let Some(error) = &self.error_message {
                    Paragraph::new(Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red)),
                        Span::styled(error, Style::default().fg(Color::Red)),
                    ]))
                } else {
                    Paragraph::new(Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red)),
                        Span::styled("Incorrect password", Style::default().fg(Color::Red)),
                    ]))
                }
            }
            UnlockState::Input => {
                if self.failed_attempts > 0 {
                    Paragraph::new(Line::from(Span::styled(
                        format!("Failed attempts: {}", self.failed_attempts),
                        Style::default().fg(Color::DarkGray),
                    )))
                } else {
                    Paragraph::new(Line::from(""))
                }
            }
        };

        frame.render_widget(status, chunks[3]);

        // Info section
        let info = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "Enter your master password to unlock the vault",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Your data is encrypted and stored locally",
                Style::default().fg(Color::DarkGray),
            )]),
        ])
        .wrap(Wrap { trim: true });

        frame.render_widget(info, chunks[5]);

        // Footer
        let footer_spans = vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if self.can_unlock() {
                ": Unlock    "
            } else {
                "           "
            }),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Quit"),
        ];

        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[6]);
    }
}

impl Default for UnlockScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_screen_new() {
        let screen = UnlockScreen::new();
        assert_eq!(screen.password(), "");
        assert_eq!(screen.state(), UnlockState::Input);
        assert_eq!(screen.failed_attempts(), 0);
        assert!(!screen.can_unlock());
    }

    #[test]
    fn test_unlock_screen_handle_char() {
        let mut screen = UnlockScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_char('c');
        assert_eq!(screen.password(), "abc");
    }

    #[test]
    fn test_unlock_screen_handle_backspace() {
        let mut screen = UnlockScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_backspace();
        assert_eq!(screen.password(), "a");
    }

    #[test]
    fn test_unlock_screen_can_unlock() {
        let mut screen = UnlockScreen::new();
        assert!(!screen.can_unlock());

        screen.handle_char('a');
        assert!(screen.can_unlock());

        screen.set_unlocking();
        assert!(!screen.can_unlock()); // Can't unlock while unlocking
    }

    #[test]
    fn test_unlock_screen_state_transitions() {
        let mut screen = UnlockScreen::new();

        // Input -> Unlocking
        screen.handle_char('t');
        screen.handle_char('e');
        screen.handle_char('s');
        screen.handle_char('t');
        screen.set_unlocking();
        assert_eq!(screen.state(), UnlockState::Unlocking);

        // Unlocking -> Failed
        screen.set_failed("Wrong password");
        assert_eq!(screen.state(), UnlockState::Failed);
        assert_eq!(screen.failed_attempts(), 1);
        assert_eq!(screen.password(), ""); // Cleared on failure

        // Failed -> Input (after typing)
        screen.handle_char('x');
        assert_eq!(screen.state(), UnlockState::Input);

        // Input -> Success
        screen.set_success();
        assert_eq!(screen.state(), UnlockState::Input);
        assert_eq!(screen.failed_attempts(), 0);
        assert_eq!(screen.password(), "");
    }

    #[test]
    fn test_unlock_screen_failed_attempts() {
        let mut screen = UnlockScreen::new();

        screen.set_failed("Wrong");
        assert_eq!(screen.failed_attempts(), 1);

        screen.set_failed("Wrong again");
        assert_eq!(screen.failed_attempts(), 2);

        screen.set_success();
        assert_eq!(screen.failed_attempts(), 0);
    }

    #[test]
    fn test_unlock_screen_clear() {
        let mut screen = UnlockScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.clear();
        assert_eq!(screen.password(), "");
    }

    #[test]
    fn test_unlock_screen_no_input_while_unlocking() {
        let mut screen = UnlockScreen::new();
        screen.handle_char('a');
        screen.set_unlocking();
        screen.handle_char('b');
        assert_eq!(screen.password(), "a"); // 'b' ignored while unlocking
    }
}
