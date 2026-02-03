//! Passkey Generation Screen
//!
//! Displays the generated 24-word Passkey and asks user to confirm they've saved it.

use crate::crypto::passkey::Passkey;
use anyhow::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Passkey generation screen
#[derive(Debug, Clone)]
pub struct PasskeyGenerateScreen {
    /// Word count (12 or 24)
    word_count: usize,
    /// The generated words
    words: Option<Vec<String>>,
    /// Whether user confirmed they saved the Passkey
    confirmed: bool,
    /// Whether Passkey was copied to clipboard
    copied: bool,
    /// Error message to display
    error: Option<String>,
}

impl PasskeyGenerateScreen {
    /// Create a new passkey generation screen
    pub fn new() -> Self {
        Self {
            word_count: 24,
            words: None,
            confirmed: false,
            copied: false,
            error: None,
        }
    }

    /// Create with specific word count
    pub fn with_word_count(word_count: usize) -> Self {
        assert!(
            word_count == 12 || word_count == 24,
            "Word count must be 12 or 24"
        );
        Self {
            word_count,
            words: None,
            confirmed: false,
            copied: false,
            error: None,
        }
    }

    /// Get the word count
    pub fn word_count(&self) -> usize {
        self.word_count
    }

    /// Check if words have been generated
    pub fn is_generated(&self) -> bool {
        self.words.is_some()
    }

    /// Get the generated words
    pub fn words(&self) -> Option<&[String]> {
        self.words.as_deref()
    }

    /// Check if user confirmed
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    /// Generate a new Passkey
    pub async fn generate(&mut self) -> Result<()> {
        let passkey = Passkey::generate(self.word_count)?;
        self.words = Some(passkey.to_words());
        self.error = None;
        Ok(())
    }

    /// Set the words directly (for testing or manual input)
    pub fn set_words(&mut self, words: Vec<String>) {
        assert!(
            words.len() == self.word_count,
            "Expected {} words",
            self.word_count
        );
        self.words = Some(words);
        self.error = None;
    }

    /// Toggle confirmation state
    pub fn toggle_confirm(&mut self) {
        if self.words.is_some() {
            self.confirmed = !self.confirmed;
        }
    }

    /// Set confirmation state directly
    pub fn set_confirmed(&mut self, confirmed: bool) {
        if self.words.is_some() {
            self.confirmed = confirmed;
        }
    }

    /// Mark as copied to clipboard
    pub fn mark_copied(&mut self) {
        self.copied = true;
    }

    /// Check if can proceed to next step
    pub fn can_proceed(&self) -> bool {
        self.words.is_some() && self.confirmed
    }

    /// Get error message
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Render the passkey generation screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(3), // Warning
                    Constraint::Min(0),    // Passkey display
                    Constraint::Length(3), // Confirmation
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "Generate New Passkey",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        // Warning message
        let warning = Paragraph::new(vec![Line::from(vec![
            Span::styled("⚠️ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "Please save these {} words, this is the ONLY way to recover your data!",
                    self.word_count
                ),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])])
        .alignment(Alignment::Center);

        frame.render_widget(warning, chunks[2]);

        // Passkey words display
        if let Some(words) = &self.words {
            let mut lines = vec![];

            // Display words in columns (4 per row for 24 words, 3 per row for 12 words)
            let cols = if self.word_count == 24 { 4 } else { 3 };

            for (idx, word) in words.iter().enumerate() {
                let row = idx / cols;
                let _col = idx % cols;

                // Ensure we have enough rows
                while lines.len() <= row {
                    lines.push(String::new());
                }

                // Format: "  1. abandon  " with spacing
                let entry = format!("{:>3}. {:<12} ", idx + 1, word);
                lines[row].push_str(&entry);
            }

            let passkey_lines: Vec<Line> = lines
                .iter()
                .map(|l| {
                    Line::from(Span::styled(
                        l.as_str(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ))
                })
                .collect();

            let passkey = Paragraph::new(passkey_lines)
                .block(Block::default().borders(Borders::ALL).title(" Passkey "))
                .wrap(Wrap { trim: false });

            frame.render_widget(passkey, chunks[3]);
        } else {
            // Not generated yet
            let loading = Paragraph::new(vec![Line::from(Span::styled(
                "Generating Passkey...",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ))])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

            frame.render_widget(loading, chunks[3]);
        }

        // Confirmation checkbox
        let confirm_text = if self.confirmed {
            vec![
                Span::styled("✓", Style::default().fg(Color::Green)),
                Span::raw(" I have saved the Passkey"),
            ]
        } else {
            vec![
                Span::styled("☐", Style::default().fg(Color::White)),
                Span::raw(" I have saved the Passkey"),
            ]
        };

        let confirmation = Paragraph::new(vec![
            Line::from(confirm_text),
            Line::from(vec![Span::styled(
                "  Data cannot be recovered if lost!",
                Style::default().fg(Color::Red),
            )]),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(confirmation, chunks[4]);

        // Footer
        let footer_spans = vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if self.can_proceed() {
                ": Next    "
            } else {
                ": Confirm first    "
            }),
            Span::styled(
                "Space",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Confirm    "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Back"),
        ];

        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[5]);
    }
}

impl Default for PasskeyGenerateScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_generate_new() {
        let screen = PasskeyGenerateScreen::new();
        assert_eq!(screen.word_count(), 24);
        assert!(!screen.is_generated());
        assert!(!screen.is_confirmed());
    }

    #[test]
    fn test_passkey_generate_with_word_count() {
        let screen = PasskeyGenerateScreen::with_word_count(12);
        assert_eq!(screen.word_count(), 12);
    }

    #[test]
    fn test_passkey_generate_set_words() {
        let mut screen = PasskeyGenerateScreen::new();
        let words = vec!["word".to_string(); 24];
        screen.set_words(words.clone());
        assert!(screen.is_generated());
        assert_eq!(screen.words(), Some(words.as_slice()));
    }

    #[test]
    fn test_passkey_generate_toggle_confirm() {
        let mut screen = PasskeyGenerateScreen::new();
        screen.set_words(vec!["word".to_string(); 24]);

        screen.toggle_confirm();
        assert!(screen.is_confirmed());

        screen.toggle_confirm();
        assert!(!screen.is_confirmed());
    }

    #[test]
    fn test_passkey_generate_can_proceed() {
        let mut screen = PasskeyGenerateScreen::new();
        assert!(!screen.can_proceed());

        screen.set_words(vec!["word".to_string(); 24]);
        assert!(!screen.can_proceed());

        screen.set_confirmed(true);
        assert!(screen.can_proceed());
    }
}
