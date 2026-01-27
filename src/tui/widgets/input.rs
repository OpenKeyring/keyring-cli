//! Command Input Widget
//!
//! Interactive command input with autocomplete support.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Command input widget state
pub struct CommandInput {
    /// Current input buffer
    buffer: String,
    /// Cursor position
    cursor: usize,
    /// Autocomplete suggestions
    suggestions: Vec<String>,
    /// Selected suggestion index
    selected_suggestion: Option<usize>,
}

impl Default for CommandInput {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandInput {
    /// Create a new command input
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            suggestions: Vec::new(),
            selected_suggestion: None,
        }
    }

    /// Get the current input buffer
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Clear the input buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
        self.suggestions.clear();
        self.selected_suggestion = None;
    }

    /// Add a character to the buffer
    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Remove character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    /// Set suggestions for autocomplete
    pub fn set_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions;
        self.selected_suggestion = if self.suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Select next suggestion
    pub fn next_suggestion(&mut self) {
        if let Some(ref mut idx) = self.selected_suggestion {
            if !self.suggestions.is_empty() {
                *idx = (*idx + 1) % self.suggestions.len();
            }
        }
    }

    /// Select previous suggestion
    pub fn prev_suggestion(&mut self) {
        if let Some(ref mut idx) = self.selected_suggestion {
            if !self.suggestions.is_empty() {
                *idx = if *idx == 0 {
                    self.suggestions.len() - 1
                } else {
                    *idx - 1
                };
            }
        }
    }

    /// Apply selected suggestion
    pub fn apply_suggestion(&mut self) -> Option<String> {
        self.selected_suggestion.and_then(|idx| {
            self.suggestions.get(idx).cloned().map(|suggestion| {
                // TODO: Implement smart replacement based on cursor position
                self.buffer = suggestion;
                self.cursor = self.buffer.len();
                self.suggestions.clear();
                self.selected_suggestion = None;
                self.buffer.clone()
            })
        })
    }

    /// Render the command input
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let input_text = if self.buffer.is_empty() {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Type /help for commands...",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ])]
        } else {
            vec![Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Gray)),
                Span::raw(&self.buffer),
            ])]
        };

        let paragraph = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Set cursor position
        frame.set_cursor_position((area.x + 2 + self.cursor as u16, area.y + 1));
    }
}
