//! Search Bar Component
//!
//! Provides a search input bar for filtering passwords.

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

/// Search bar component
pub struct SearchBar {
    /// Component ID
    id: ComponentId,
    /// Search query
    query: String,
    /// Visibility state
    visible: bool,
    /// Cursor position
    cursor_position: usize,
}

impl SearchBar {
    /// Create a new search bar
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            query: String::new(),
            visible: false,
            cursor_position: 0,
        }
    }

    /// Show the search bar
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.cursor_position = 0;
    }

    /// Hide the search bar and clear query
    pub fn hide(&mut self) {
        self.visible = false;
        self.query.clear();
        self.cursor_position = 0;
    }

    /// Check if the search bar is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the current search query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Handle key event
    pub fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if !self.visible {
            return HandleResult::Ignored;
        }

        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                self.hide();
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                // Keep query but hide bar
                self.visible = false;
                HandleResult::Consumed
            }
            KeyCode::Char(c) => {
                self.query.insert(self.cursor_position, c);
                self.cursor_position += 1;
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.query.remove(self.cursor_position);
                    HandleResult::NeedsRender
                } else {
                    HandleResult::Ignored
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.query.len() {
                    self.query.remove(self.cursor_position);
                    HandleResult::NeedsRender
                } else {
                    HandleResult::Ignored
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.query.len() {
                    self.cursor_position += 1;
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            _ => HandleResult::Ignored,
        }
    }

    /// Render the search bar
    pub fn render_frame(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Search ");

        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Build display: query + cursor
        let display = if self.query.is_empty() {
            Span::styled("Type to search...", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw(self.query.clone())
        };

        let paragraph = Paragraph::new(Line::from(display));
        frame.render_widget(paragraph, inner);
    }
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SearchBar {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl Interactive for SearchBar {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        self.handle_key(key)
    }
}

impl Render for SearchBar {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Search ");
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
    fn test_search_bar_new() {
        let bar = SearchBar::new();
        assert!(!bar.is_visible());
        assert_eq!(bar.query(), "");
    }

    #[test]
    fn test_search_bar_show_hide() {
        let mut bar = SearchBar::new();
        bar.show();
        assert!(bar.is_visible());

        bar.hide();
        assert!(!bar.is_visible());
        assert_eq!(bar.query(), "");
    }

    #[test]
    fn test_search_bar_input() {
        let mut bar = SearchBar::new();
        bar.show();

        bar.handle_key(create_key(KeyCode::Char('t')));
        bar.handle_key(create_key(KeyCode::Char('e')));
        bar.handle_key(create_key(KeyCode::Char('s')));
        bar.handle_key(create_key(KeyCode::Char('t')));

        assert_eq!(bar.query(), "test");
    }

    #[test]
    fn test_search_bar_backspace() {
        let mut bar = SearchBar::new();
        bar.show();
        bar.handle_key(create_key(KeyCode::Char('h')));
        bar.handle_key(create_key(KeyCode::Char('i')));
        bar.handle_key(create_key(KeyCode::Backspace));
        assert_eq!(bar.query(), "h");
    }

    #[test]
    fn test_search_bar_esc_closes() {
        let mut bar = SearchBar::new();
        bar.show();
        bar.handle_key(create_key(KeyCode::Char('t')));
        bar.handle_key(create_key(KeyCode::Esc));
        assert!(!bar.is_visible());
    }

    #[test]
    fn test_search_bar_enter_keeps_query() {
        let mut bar = SearchBar::new();
        bar.show();
        bar.handle_key(create_key(KeyCode::Char('t')));
        bar.handle_key(create_key(KeyCode::Char('e')));
        bar.handle_key(create_key(KeyCode::Char('s')));
        bar.handle_key(create_key(KeyCode::Char('t')));
        bar.handle_key(create_key(KeyCode::Enter));

        assert!(!bar.is_visible());
        assert_eq!(bar.query(), "test");
    }

    #[test]
    fn test_search_bar_cursor_navigation() {
        let mut bar = SearchBar::new();
        bar.show();
        bar.handle_key(create_key(KeyCode::Char('a')));
        bar.handle_key(create_key(KeyCode::Char('b')));
        bar.handle_key(create_key(KeyCode::Char('c')));

        assert_eq!(bar.query(), "abc");
        assert_eq!(bar.cursor_position, 3);

        // Move left
        bar.handle_key(create_key(KeyCode::Left));
        assert_eq!(bar.cursor_position, 2);

        // Delete at cursor (deletes 'c' since cursor is at position 2 which is 'c')
        bar.handle_key(create_key(KeyCode::Delete));
        assert_eq!(bar.query(), "ab");
    }
}
