//! Tag Editor Component
//!
//! Provides inline tag editing with autocomplete suggestions.

use crate::tui::error::TuiResult;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Tag editor component
pub struct TagEditor {
    id: ComponentId,
    /// Current tags
    tags: Vec<String>,
    /// Input buffer
    input: String,
    /// Whether focused
    focused: bool,
    /// Autocomplete suggestions
    suggestions: Vec<String>,
    /// Currently selected suggestion index
    selected_suggestion: Option<usize>,
    /// All available tags (for suggestions)
    all_tags: Vec<String>,
}

impl TagEditor {
    /// Create a new tag editor
    pub fn new(tags: Vec<String>) -> Self {
        Self {
            id: ComponentId::new(0),
            tags,
            input: String::new(),
            focused: false,
            suggestions: Vec::new(),
            selected_suggestion: None,
            all_tags: Vec::new(),
        }
    }

    /// Set all available tags for suggestions
    pub fn set_all_tags(&mut self, all_tags: Vec<String>) {
        self.all_tags = all_tags;
    }

    /// Get current tags
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Take tags (move out)
    pub fn take_tags(self) -> Vec<String> {
        self.tags
    }

    /// Set tags
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    /// Update suggestions based on current input
    fn update_suggestions(&mut self) {
        if self.input.is_empty() {
            self.suggestions.clear();
            self.selected_suggestion = None;
        } else {
            let input_lower = self.input.to_lowercase();
            self.suggestions = self
                .all_tags
                .iter()
                .filter(|t| {
                    t.to_lowercase().contains(&input_lower) && !self.tags.contains(t)
                })
                .cloned()
                .take(5) // Limit suggestions
                .collect();
            self.selected_suggestion = if self.suggestions.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char(',') | KeyCode::Enter => {
                let tag = self.input.trim().to_string();
                if !tag.is_empty() && !self.tags.contains(&tag) {
                    self.tags.push(tag);
                }
                self.input.clear();
                self.suggestions.clear();
                self.selected_suggestion = None;
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                if self.input.is_empty() && !self.tags.is_empty() {
                    self.tags.pop();
                    HandleResult::Consumed
                } else if !self.input.is_empty() {
                    self.input.pop();
                    self.update_suggestions();
                    HandleResult::NeedsRender
                } else {
                    HandleResult::Ignored
                }
            }
            KeyCode::Tab => {
                // Accept first suggestion
                if let Some(first) = self.suggestions.first().cloned() {
                    self.tags.push(first);
                    self.input.clear();
                    self.suggestions.clear();
                    self.selected_suggestion = None;
                    return HandleResult::Consumed;
                }
                HandleResult::Ignored
            }
            KeyCode::Up => {
                if self.selected_suggestion.is_some() {
                    self.selected_suggestion = self.selected_suggestion.map(|i| i.saturating_sub(1));
                    return HandleResult::Consumed;
                }
                HandleResult::Ignored
            }
            KeyCode::Down => {
                if !self.suggestions.is_empty() {
                    self.selected_suggestion = self.selected_suggestion.map(|i| {
                        (i + 1).min(self.suggestions.len() - 1)
                    });
                    return HandleResult::Consumed;
                }
                HandleResult::Ignored
            }
            KeyCode::Char(c) if c != ',' => {
                self.input.push(c);
                self.update_suggestions();
                HandleResult::NeedsRender
            }
            _ => HandleResult::Ignored,
        }
    }

    /// Render the tag editor
    pub fn render_frame(&self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(" Tags ");

        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Build tag display
        let mut spans = Vec::new();

        // Existing tags
        for tag in &self.tags {
            spans.push(Span::styled(
                format!("[{}]", tag),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(" "));
        }

        // Input cursor
        if self.focused {
            spans.push(Span::styled(
                format!("{}_", self.input),
                Style::default().fg(Color::Yellow),
            ));
        } else if self.tags.is_empty() {
            spans.push(Span::styled(
                "Press Enter to add tags...",
                Style::default().fg(Color::DarkGray),
            ));
        }

        let paragraph = Paragraph::new(Line::from(spans));
        frame.render_widget(paragraph, inner);

        // Render suggestions popup if visible
        if !self.suggestions.is_empty() && self.focused {
            self.render_suggestions(frame, area);
        }
    }

    fn render_suggestions(&self, frame: &mut Frame, _area: Rect) {
        // TODO: Render suggestions popup above the input
        // For now, suggestions are just shown as part of the UI state
        let _ = self.suggestions;
    }
}

impl Default for TagEditor {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Component for TagEditor {
    fn id(&self) -> ComponentId {
        self.id
    }
    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focused = true;
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focused = false;
        // Commit any pending input
        let tag = self.input.trim().to_string();
        if !tag.is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self.input.clear();
        self.suggestions.clear();
        Ok(())
    }
}

impl Interactive for TagEditor {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        self.handle_key(key)
    }
}

impl Render for TagEditor {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title(" Tags ");
        block.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn create_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_tag_editor_new() {
        let editor = TagEditor::new(vec!["work".to_string()]);
        assert_eq!(editor.tags().len(), 1);
        assert_eq!(editor.tags()[0], "work");
    }

    #[test]
    fn test_tag_editor_add_tag() {
        let mut editor = TagEditor::new(Vec::new());
        editor.focused = true;

        editor.handle_key(create_key(KeyCode::Char('e')));
        editor.handle_key(create_key(KeyCode::Char('m')));
        editor.handle_key(create_key(KeyCode::Char('a')));
        editor.handle_key(create_key(KeyCode::Char('i')));
        editor.handle_key(create_key(KeyCode::Char('l')));
        editor.handle_key(create_key(KeyCode::Enter));

        assert_eq!(editor.tags().len(), 1);
        assert_eq!(editor.tags()[0], "email");
    }

    #[test]
    fn test_tag_editor_backspace_remove_last() {
        let mut editor = TagEditor::new(vec!["a".to_string(), "b".to_string()]);
        editor.focused = true;

        // Backspace on empty input removes last tag
        editor.handle_key(create_key(KeyCode::Backspace));
        assert_eq!(editor.tags().len(), 1);
        assert_eq!(editor.tags()[0], "a");
    }

    #[test]
    fn test_tag_editor_no_duplicate() {
        let mut editor = TagEditor::new(vec!["work".to_string()]);
        editor.focused = true;

        editor.handle_key(create_key(KeyCode::Char('w')));
        editor.handle_key(create_key(KeyCode::Char('o')));
        editor.handle_key(create_key(KeyCode::Char('r')));
        editor.handle_key(create_key(KeyCode::Char('k')));
        editor.handle_key(create_key(KeyCode::Enter));

        // Should not add duplicate
        assert_eq!(editor.tags().len(), 1);
    }

    #[test]
    fn test_tag_editor_comma_separator() {
        let mut editor = TagEditor::new(Vec::new());
        editor.focused = true;

        editor.handle_key(create_key(KeyCode::Char('t')));
        editor.handle_key(create_key(KeyCode::Char('a')));
        editor.handle_key(create_key(KeyCode::Char('g')));
        editor.handle_key(create_key(KeyCode::Char(',')));

        assert_eq!(editor.tags().len(), 1);
        assert_eq!(editor.tags()[0], "tag");
        assert!(editor.input.is_empty());
    }

    #[test]
    fn test_tag_editor_autocomplete() {
        let mut editor = TagEditor::new(Vec::new());
        editor.focused = true;
        editor.set_all_tags(vec![
            "work".to_string(),
            "personal".to_string(),
            "finance".to_string(),
        ]);

        // Type 'f' should show 'finance' suggestion
        editor.handle_key(create_key(KeyCode::Char('f')));

        assert_eq!(editor.suggestions.len(), 1);
        assert_eq!(editor.suggestions[0], "finance");

        // Tab to accept suggestion
        editor.handle_key(create_key(KeyCode::Tab));

        assert_eq!(editor.tags().len(), 1);
        assert_eq!(editor.tags()[0], "finance");
    }

    #[test]
    fn test_tag_editor_navigation() {
        let mut editor = TagEditor::new(Vec::new());
        editor.focused = true;
        editor.set_all_tags(vec![
            "work".to_string(),
            "personal".to_string(),
            "finance".to_string(),
            "social".to_string(),
        ]);

        // Type 'a' should match multiple
        editor.handle_key(create_key(KeyCode::Char('a')));

        // Should have at least some suggestions
        assert!(!editor.suggestions.is_empty());

        // Navigate down
        editor.handle_key(create_key(KeyCode::Down));

        // Navigate up
        editor.handle_key(create_key(KeyCode::Up));
    }
}
