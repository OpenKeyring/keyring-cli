//! Password Display Popup Widget
//!
//! Shows passwords in a secure popup with auto-clear functionality.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Password popup widget
pub struct PasswordPopup {
    /// The password to display (redacted by default)
    password: String,
    /// Whether to show the actual password
    revealed: bool,
    /// Clipboard timeout in seconds
    timeout_seconds: u64,
}

impl PasswordPopup {
    /// Create a new password popup
    pub fn new(password: String) -> Self {
        Self {
            password,
            revealed: false,
            timeout_seconds: 30,
        }
    }

    /// Set clipboard timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Toggle password visibility
    pub fn toggle_reveal(&mut self) {
        self.revealed = !self.revealed;
    }

    /// Render the popup
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear area behind popup
        frame.render_widget(Clear, area);

        // Create popup layout
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),  // Title
                    Constraint::Length(3),  // Password
                    Constraint::Length(2),  // Instructions
                ]
                .as_ref(),
            )
            .margin(1)
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("🔑 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Password",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(title, popup_chunks[0]);

        // Password (revealed or redacted)
        let display_text = if self.revealed {
            self.password.clone()
        } else {
            "•".repeat(self.password.chars().count())
        };

        let password_paragraph = Paragraph::new(Line::from(vec![Span::styled(
            display_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(password_paragraph, popup_chunks[1]);

        // Instructions
        let instructions = vec![
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Space",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to reveal/hide", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" to copy ({}s timeout)", self.timeout_seconds),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "q",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close", Style::default().fg(Color::Gray)),
            ]),
        ];

        let instructions_paragraph = Paragraph::new(instructions)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(instructions_paragraph, popup_chunks[2]);
    }
}
