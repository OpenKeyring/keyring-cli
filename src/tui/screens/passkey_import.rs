//! Passkey Import Screen
//!
//! Allows users to import an existing Passkey by entering their mnemonic words.

use anyhow::{anyhow, Result};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Passkey import screen
#[derive(Debug, Clone)]
pub struct PasskeyImportScreen {
    /// User input buffer
    input: String,
    /// Whether the input has been validated
    validated: bool,
    /// Validation error message
    validation_error: Option<String>,
    /// The validated words (if successful)
    words: Option<Vec<String>>,
}

impl PasskeyImportScreen {
    /// Create a new passkey import screen
    pub fn new() -> Self {
        Self {
            input: String::new(),
            validated: false,
            validation_error: None,
            words: None,
        }
    }

    /// Get current input
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Check if input has been validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }

    /// Get validation error
    pub fn validation_error(&self) -> Option<&str> {
        self.validation_error.as_deref()
    }

    /// Get the validated words
    pub fn words(&self) -> Option<&[String]> {
        self.words.as_deref()
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if !self.validated && !c.is_control() {
            self.input.push(c);
            self.validation_error = None;
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if !self.validated {
            self.input.pop();
            self.validation_error = None;
        }
    }

    /// Clear input
    pub fn clear(&mut self) {
        self.input.clear();
        self.validated = false;
        self.validation_error = None;
        self.words = None;
    }

    /// Validate the input as a BIP39 mnemonic
    pub fn validate(&mut self) -> Result<()> {
        use crate::crypto::passkey::Passkey;

        // Split into words
        let words: Vec<String> = self.input.split_whitespace().map(String::from).collect();

        // Check word count
        if words.len() != 12 && words.len() != 24 {
            self.validation_error = Some(format!(
                "Passkey must be 12 or 24 words (current: {} words)",
                words.len()
            ));
            return Err(anyhow!("{}", self.validation_error.as_ref().unwrap()));
        }

        // Validate BIP39 checksum
        Passkey::from_words(&words).map_err(|e| {
            self.validation_error = Some(format!("Invalid Passkey: {}", e));
            anyhow!("{}", self.validation_error.as_ref().unwrap())
        })?;

        // Success
        self.validated = true;
        self.words = Some(words);
        self.validation_error = None;
        Ok(())
    }

    /// Check if can proceed to next step
    pub fn can_proceed(&self) -> bool {
        self.validated && self.words.is_some()
    }

    /// Render the passkey import screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(2), // Instructions
                    Constraint::Length(5), // Input area
                    Constraint::Length(2), // Error/status
                    Constraint::Min(0),    // Spacer
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "Import Existing Passkey",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(vec![Line::from(Span::styled(
            "Enter your 12 or 24 word Passkey (separated by spaces):",
            Style::default().fg(Color::White),
        ))])
        .alignment(Alignment::Left);

        frame.render_widget(instructions, chunks[2]);

        // Input area
        let input_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if self.input.is_empty() {
                        "Type Passkey here..."
                    } else {
                        &self.input
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Hint: Press Enter to validate when done",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input "),
        )
        .wrap(Wrap { trim: true });

        frame.render_widget(input_paragraph, chunks[3]);

        // Status/Error area
        let status_paragraph = if let Some(error) = &self.validation_error {
            Paragraph::new(Line::from(vec![
                Span::styled("✗ ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]))
        } else if self.validated {
            Paragraph::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::styled("Passkey validated successfully", Style::default().fg(Color::Green)),
            ]))
        } else {
            Paragraph::new(Line::from(""))
        };

        frame.render_widget(status_paragraph, chunks[4]);

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
                ": Validate    "
            }),
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

        frame.render_widget(footer, chunks[6]);
    }
}

impl Default for PasskeyImportScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::tui::traits::Interactive for PasskeyImportScreen {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> crate::tui::traits::HandleResult {
        use crossterm::event::KeyCode;
        use crate::tui::traits::HandleResult;

        match key.code {
            KeyCode::Char(c) => {
                self.handle_char(c);
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.can_proceed() {
                    HandleResult::Consumed
                } else {
                    // Try to validate
                    let _ = self.validate();
                    HandleResult::NeedsRender
                }
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl crate::tui::traits::WizardStepValidator for PasskeyImportScreen {
    fn validate_step(&self) -> bool {
        self.can_proceed()
    }

    fn validation_error(&self) -> Option<String> {
        self.validation_error.clone()
    }

    fn sync_to_state(&self, state: &mut crate::tui::screens::wizard::WizardState) {
        if let Some(words) = &self.words {
            state.set_passkey_words(words.clone());
        }
    }

    fn load_from_state(&mut self, state: &crate::tui::screens::wizard::WizardState) {
        if let Some(words) = state.require_passkey_words() {
            self.input = words.join(" ");
            self.words = Some(words.to_vec());
            self.validated = true;
        }
    }

    fn clear_input(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_import_new() {
        let screen = PasskeyImportScreen::new();
        assert_eq!(screen.input(), "");
        assert!(!screen.is_validated());
    }

    #[test]
    fn test_passkey_import_handle_char() {
        let mut screen = PasskeyImportScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_char('c');
        assert_eq!(screen.input(), "abc");
    }

    #[test]
    fn test_passkey_import_handle_backspace() {
        let mut screen = PasskeyImportScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_backspace();
        assert_eq!(screen.input(), "a");
    }

    #[test]
    fn test_passkey_import_clear() {
        let mut screen = PasskeyImportScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.clear();
        assert_eq!(screen.input(), "");
        assert!(!screen.is_validated());
    }

    #[test]
    fn test_passkey_import_validate_wrong_count() {
        let mut screen = PasskeyImportScreen::new();
        screen.input = "one two three".to_string();

        let result = screen.validate();
        assert!(result.is_err());
        assert!(screen.validation_error().unwrap().contains("12 or 24 words"));
    }
}
