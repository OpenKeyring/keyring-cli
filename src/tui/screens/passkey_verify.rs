//! Passkey Verification Screen
//!
//! Verify user has correctly saved their passkey by asking for 3 random positions

use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Passkey verification screen
#[derive(Debug)]
pub struct PasskeyVerifyScreen {
    /// The 24-word passkey
    passkey_words: Vec<String>,
    /// 3 random positions to verify (1-indexed)
    positions: [usize; 3],
    /// User input for each position
    inputs: [String; 3],
    /// Currently focused input (0, 1, or 2)
    focused: usize,
    /// Error message
    error: Option<String>,
    /// Component ID
    id: ComponentId,
}

impl PasskeyVerifyScreen {
    /// Create new verification screen with random positions
    pub fn new(passkey_words: Vec<String>) -> Self {
        // Generate 3 unique random positions (1-24)
        let mut rng = rand::rng();
        let mut positions = [0usize; 3];

        for i in 0..3 {
            loop {
                let pos = rng.random_range(1..=24);
                if !positions.contains(&pos) {
                    positions[i] = pos;
                    break;
                }
            }
        }

        Self {
            passkey_words,
            positions,
            inputs: [String::new(), String::new(), String::new()],
            focused: 0,
            error: None,
            id: ComponentId::new(3012),
        }
    }

    /// Create with specific positions (for testing)
    pub fn with_positions(passkey_words: Vec<String>, positions: [usize; 3]) -> Self {
        Self {
            passkey_words,
            positions,
            inputs: [String::new(), String::new(), String::new()],
            focused: 0,
            error: None,
            id: ComponentId::new(3012),
        }
    }

    /// Get the random positions being verified
    pub fn positions(&self) -> [usize; 3] {
        self.positions
    }

    /// Verify if all inputs match
    pub fn verify(&self) -> bool {
        for (i, &pos) in self.positions.iter().enumerate() {
            if self.inputs[i].to_lowercase().trim()
                != self.passkey_words[pos - 1].to_lowercase().trim()
            {
                return false;
            }
        }
        true
    }

    /// Get the expected word at position
    pub fn expected_word(&self, pos: usize) -> &str {
        &self.passkey_words[self.positions[pos] - 1]
    }

    /// Get user inputs
    pub fn inputs(&self) -> &[String; 3] {
        &self.inputs
    }

    /// Get the currently focused field index
    pub fn focused(&self) -> usize {
        self.focused
    }

    /// Set error message
    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Clear all inputs
    pub fn clear_inputs(&mut self) {
        self.inputs = [String::new(), String::new(), String::new()];
        self.error = None;
    }
}

impl Render for PasskeyVerifyScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("Verify Your Passkey")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = vec![
            Line::from(""),
            Line::from("Please enter the following words from your passkey:")
                .style(Style::default().fg(Color::White)),
            Line::from(""),
        ];

        // Add input fields for each position
        for i in 0..3 {
            let pos = self.positions[i];
            let label = format!("Word #{}: ", pos);
            let input = &self.inputs[i];
            let focused = i == self.focused;

            let style = if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(label, Style::default().fg(Color::Cyan)),
                Span::styled(input.clone(), style),
                if focused {
                    Span::styled("_", Style::default().fg(Color::Yellow))
                } else {
                    Span::raw("")
                },
            ]));
        }

        // Error message if any
        if let Some(error) = &self.error {
            lines.push(Line::from(""));
            lines.push(
                Line::from(format!("Error: {}", error)).style(Style::default().fg(Color::Red)),
            );
        }

        // Instructions
        lines.push(Line::from(""));
        lines.push(
            Line::from("TAB: Switch field  |  ENTER: Submit  |  ESC: Back")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        paragraph.render(inner, buf);
    }
}

impl Interactive for PasskeyVerifyScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Tab => {
                self.focused = (self.focused + 1) % 3;
                HandleResult::NeedsRender
            }
            KeyCode::BackTab => {
                self.focused = if self.focused == 0 {
                    2
                } else {
                    self.focused - 1
                };
                HandleResult::NeedsRender
            }
            KeyCode::Char(c) => {
                self.inputs[self.focused].push(c);
                self.error = None;
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                self.inputs[self.focused].pop();
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.verify() {
                    HandleResult::Consumed
                } else {
                    self.error =
                        Some("One or more words are incorrect. Please try again.".to_string());
                    HandleResult::NeedsRender
                }
            }
            KeyCode::Esc => HandleResult::Ignored,
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for PasskeyVerifyScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}
