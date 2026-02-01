//! Passkey Confirmation Screen
//!
//! Shows a summary of the generated Passkey and asks user to confirm they've saved it.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Passkey confirmation screen
#[derive(Debug, Clone)]
pub struct PasskeyConfirmScreen {
    /// The Passkey words to display
    passkey_words: Vec<String>,
    /// Whether user confirmed they saved the Passkey
    confirmed: bool,
}

impl PasskeyConfirmScreen {
    /// Create a new confirmation screen with the given words
    pub fn new(words: Vec<String>) -> Self {
        Self {
            passkey_words: words,
            confirmed: false,
        }
    }

    /// Get the Passkey words
    pub fn words(&self) -> &[String] {
        &self.passkey_words
    }

    /// Check if user confirmed
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    /// Toggle confirmation state
    pub fn toggle(&mut self) {
        self.confirmed = !self.confirmed;
    }

    /// Set confirmation state directly
    pub fn set_confirmed(&mut self, confirmed: bool) {
        self.confirmed = confirmed;
    }

    /// Check if can proceed
    pub fn can_proceed(&self) -> bool {
        self.confirmed
    }

    /// Render the confirmation screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),  // Title
                    Constraint::Length(2),  // Spacer
                    Constraint::Length(2),  // Warning
                    Constraint::Min(0),     // Passkey summary
                    Constraint::Length(3),  // Confirmation
                    Constraint::Length(3),  // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title
        let title = Paragraph::new(vec![
            Line::from(Span::styled(
                "确认 Passkey",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        // Warning message
        let warning = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("⚠️ ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "请确认您已妥善保存 Passkey",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ])
        .alignment(Alignment::Center);

        frame.render_widget(warning, chunks[2]);

        // Passkey summary (first 4 and last 4 words)
        let word_count = self.passkey_words.len();
        let display_count = 4;

        let mut summary_lines = vec![
            Line::from(Span::styled(
                format!("Passkey 摘要 (共 {} 词):", word_count),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        // First 4 words
        summary_lines.push(Line::from(Span::styled(
            "前 4 词:",
            Style::default().fg(Color::Gray),
        )));
        let mut first_line = Vec::new();
        for word in self.passkey_words.iter().take(display_count) {
            first_line.push(Span::styled(
                format!("{} ", word),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ));
        }
        summary_lines.push(Line::from(first_line));

        summary_lines.push(Line::from(""));

        // Last 4 words
        summary_lines.push(Line::from(Span::styled(
            "后 4 词:".to_string(),
            Style::default().fg(Color::Gray),
        )));
        let mut last_line = Vec::new();
        for word in self.passkey_words.iter().skip(word_count.saturating_sub(display_count)) {
            last_line.push(Span::styled(
                format!("{} ", word),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ));
        }
        summary_lines.push(Line::from(last_line));

        let summary = Paragraph::new(summary_lines)
            .block(Block::default().borders(Borders::ALL).title(" Passkey 摘要 "));

        frame.render_widget(summary, chunks[3]);

        // Confirmation checkbox
        let confirm_text = if self.confirmed {
            vec![
                Span::styled("✓", Style::default().fg(Color::Green)),
                Span::raw(" 我已安全保存此 Passkey"),
            ]
        } else {
            vec![
                Span::styled("☐", Style::default().fg(Color::White)),
                Span::raw(" 我已安全保存此 Passkey"),
            ]
        };

        let confirmation = Paragraph::new(vec![
            Line::from(confirm_text),
            Line::from(vec![
                Span::styled("  丢失将无法恢复数据！", Style::default().fg(Color::Red)),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(confirmation, chunks[4]);

        // Footer
        let footer_spans = vec![
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(if self.can_proceed() {
                ": 下一步    "
            } else {
                ": 需先确认    "
            }),
            Span::styled("Space", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(": 确认    "),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(": 返回"),
        ];

        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[5]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_confirm_new() {
        let words = vec!["word".to_string(); 24];
        let screen = PasskeyConfirmScreen::new(words);
        assert!(!screen.is_confirmed());
    }

    #[test]
    fn test_passkey_confirm_toggle() {
        let words = vec!["word".to_string(); 24];
        let mut screen = PasskeyConfirmScreen::new(words);

        screen.toggle();
        assert!(screen.is_confirmed());

        screen.toggle();
        assert!(!screen.is_confirmed());
    }

    #[test]
    fn test_passkey_confirm_can_proceed() {
        let words = vec!["word".to_string(); 24];
        let mut screen = PasskeyConfirmScreen::new(words);

        assert!(!screen.can_proceed());
        screen.set_confirmed(true);
        assert!(screen.can_proceed());
    }
}
