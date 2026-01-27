//! Mnemonic Display Widget
//!
//! Shows BIP39 mnemonic phrases in a secure popup.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Mnemonic display widget
pub struct MnemonicDisplay {
    /// The mnemonic words
    words: Vec<String>,
}

impl MnemonicDisplay {
    /// Create a new mnemonic display
    pub fn new(words: Vec<String>) -> Self {
        Self { words }
    }

    /// Create from a space-separated mnemonic string
    pub fn from_str(mnemonic: &str) -> Self {
        Self {
            words: mnemonic.split_whitespace().map(String::from).collect(),
        }
    }

    /// Render the mnemonic display
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear area behind popup
        frame.render_widget(Clear, area);

        // Create popup layout
        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Min(1),    // Mnemonic words
                    Constraint::Length(2), // Instructions
                ]
                .as_ref(),
            )
            .margin(1)
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("🔑 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("Recovery Key ({} words)", self.words.len()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);

        frame.render_widget(title, popup_chunks[0]);

        // Mnemonic words (display in columns)
        let words_text: Vec<Line> = self
            .words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let word_num = i + 1;
                Line::from(vec![
                    Span::styled(
                        format!("{:2}. ", word_num),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        word,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            })
            .collect();

        let words_paragraph = Paragraph::new(words_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        frame.render_widget(words_paragraph, popup_chunks[1]);

        // Instructions
        let instructions = Line::from(vec![
            Span::styled("⚠️  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Save this key securely. It will not be shown again.",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let instructions_paragraph = Paragraph::new(instructions).alignment(Alignment::Center);

        frame.render_widget(instructions_paragraph, popup_chunks[2]);
    }
}
