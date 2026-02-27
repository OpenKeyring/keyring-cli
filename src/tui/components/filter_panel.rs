//! Filter Panel Component
//!
//! A vertical list of filter options for password entries.
//! Supports keyboard navigation (j/k) and toggle (Enter/Space).

use crate::tui::error::TuiResult;
use crate::tui::state::filter_state::{FilterState, FilterType};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Filter item representation
#[derive(Debug, Clone)]
pub struct FilterItem {
    /// Filter type
    pub filter_type: FilterType,
    /// Display label (e.g., "Trash", "Expired", "Favorite")
    pub label: String,
    /// Icon/emoji for visual representation
    pub icon: String,
    /// Count of matching entries (set by parent)
    pub count: usize,
}

impl FilterItem {
    /// Create a new filter item
    pub fn new(filter_type: FilterType, label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            filter_type,
            label: label.into(),
            icon: icon.into(),
            count: 0,
        }
    }

    /// Set the count of matching entries
    #[must_use]
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

impl Default for FilterItem {
    fn default() -> Self {
        Self::new(FilterType::Trash, "Trash", "🗑")
    }
}

/// Filter panel component
///
/// Displays a list of filter options with keyboard navigation.
/// Users can toggle filters on/off to filter the password list.
pub struct FilterPanel {
    /// Component ID
    id: ComponentId,
    /// Filter items
    items: Vec<FilterItem>,
    /// Currently highlighted index (for keyboard navigation)
    highlighted_index: usize,
    /// Whether the panel has focus
    focused: bool,
}

impl FilterPanel {
    /// Create a new filter panel with default filter items
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            items: Self::create_default_items(),
            highlighted_index: 0,
            focused: false,
        }
    }

    /// Create filter panel with custom items
    pub fn with_items(items: Vec<FilterItem>) -> Self {
        Self {
            id: ComponentId::new(0),
            items,
            highlighted_index: 0,
            focused: false,
        }
    }

    /// Set component ID
    #[must_use]
    pub fn with_id(mut self, id: ComponentId) -> Self {
        self.id = id;
        self
    }

    /// Create default filter items
    fn create_default_items() -> Vec<FilterItem> {
        vec![
            FilterItem::new(FilterType::Trash, "Trash", "🗑"),
            FilterItem::new(FilterType::Expired, "Expired", "⏰"),
            FilterItem::new(FilterType::Favorite, "Favorite", "⭐"),
        ]
    }

    /// Move selection up
    fn move_up(&mut self) {
        if !self.items.is_empty() {
            if self.highlighted_index > 0 {
                self.highlighted_index -= 1;
            } else {
                // Wrap to bottom
                self.highlighted_index = self.items.len() - 1;
            }
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        if !self.items.is_empty() {
            if self.highlighted_index < self.items.len() - 1 {
                self.highlighted_index += 1;
            } else {
                // Wrap to top
                self.highlighted_index = 0;
            }
        }
    }

    /// Get currently highlighted item
    pub fn highlighted_item(&self) -> Option<&FilterItem> {
        self.items.get(self.highlighted_index)
    }

    /// Check if the panel is currently focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Toggle the currently highlighted filter
    fn toggle_highlighted(&mut self, state: &mut FilterState) {
        if let Some(item) = self.items.get(self.highlighted_index) {
            state.toggle(item.filter_type);
        }
    }

    /// Update item counts based on filter state
    pub fn update_counts(&mut self, counts: impl IntoIterator<Item = (FilterType, usize)>) {
        for (filter_type, count) in counts {
            if let Some(item) = self.items.iter_mut().find(|i| i.filter_type == filter_type) {
                item.count = count;
            }
        }
    }

    /// Render to frame (preferred method)
    pub fn render_frame(&self, frame: &mut Frame, area: Rect, state: &FilterState) {
        if area.height < 3 {
            // Not enough space to render
            return;
        }

        // Create border block
        let border_style = if self.focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Filters ");

        let inner_area = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Render filter items
        let items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_active = state.is_active(&item.filter_type);
                let is_highlighted = index == self.highlighted_index && self.focused;

                let style = if is_highlighted {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else if is_active {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Format: "[x] 🗑 Trash (5)"
                let check_mark = if is_active { "[x]" } else { "[ ]" };
                let count_text = if item.count > 0 {
                    format!(" ({})", item.count)
                } else {
                    String::new()
                };

                let text = format!("{} {} {}{}", check_mark, item.icon, item.label, count_text);
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner_area);
    }

    /// Handle key event with state mutation
    pub fn handle_key_with_state(&mut self, key: KeyEvent, state: &mut FilterState) -> HandleResult {
        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                HandleResult::Consumed
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                HandleResult::Consumed
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_highlighted(state);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Default for FilterPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for FilterPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Simplified render without state - just show placeholder
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Filters ");
        let inner = block.inner(area);
        block.render(area, buf);

        // Show items without active state
        for (i, item) in self.items.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let text = format!("{} {}", item.icon, item.label);
            let paragraph = Paragraph::new(text);
            paragraph.render(
                Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
                buf,
            );
        }
    }
}

impl Interactive for FilterPanel {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                HandleResult::Consumed
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for FilterPanel {
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_panel_creation() {
        let panel = FilterPanel::new();
        assert_eq!(panel.items.len(), 3);
        assert_eq!(panel.highlighted_index, 0);
        assert!(!panel.focused);
    }

    #[test]
    fn test_filter_item_creation() {
        let item = FilterItem::new(FilterType::Trash, "Test", "X");
        assert_eq!(item.label, "Test");
        assert_eq!(item.icon, "X");
        assert_eq!(item.count, 0);
    }

    #[test]
    fn test_navigation_down() {
        let mut panel = FilterPanel::new();

        panel.handle_key(KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()));
        assert_eq!(panel.highlighted_index, 1);

        panel.handle_key(KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()));
        assert_eq!(panel.highlighted_index, 2);

        // Wrap to top
        panel.handle_key(KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()));
        assert_eq!(panel.highlighted_index, 0);
    }

    #[test]
    fn test_navigation_up() {
        let mut panel = FilterPanel::new();
        panel.highlighted_index = 1;

        panel.handle_key(KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty()));
        assert_eq!(panel.highlighted_index, 0);

        // Wrap to bottom
        panel.handle_key(KeyEvent::new(KeyCode::Char('k'), crossterm::event::KeyModifiers::empty()));
        assert_eq!(panel.highlighted_index, 2);
    }

    #[test]
    fn test_toggle_filter() {
        let mut panel = FilterPanel::new();
        let mut state = FilterState::new();

        // Toggle first item (Trash)
        panel.toggle_highlighted(&mut state);
        assert!(state.is_active(&FilterType::Trash));

        // Toggle again to deactivate
        panel.toggle_highlighted(&mut state);
        assert!(!state.is_active(&FilterType::Trash));
    }

    #[test]
    fn test_handle_key_with_state() {
        let mut panel = FilterPanel::new();
        let mut state = FilterState::new();

        // Toggle with Enter
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert!(state.is_active(&FilterType::Trash));

        // Navigate down
        let result = panel.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('j'), crossterm::event::KeyModifiers::empty()),
            &mut state,
        );
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(panel.highlighted_index, 1);
    }

    #[test]
    fn test_update_counts() {
        let mut panel = FilterPanel::new();
        panel.update_counts([
            (FilterType::Trash, 5),
            (FilterType::Expired, 3),
            (FilterType::Favorite, 10),
        ]);

        assert_eq!(panel.items[0].count, 5);
        assert_eq!(panel.items[1].count, 3);
        assert_eq!(panel.items[2].count, 10);
    }

    #[test]
    fn test_highlighted_item() {
        let panel = FilterPanel::new();
        let item = panel.highlighted_item();
        assert!(item.is_some());
        assert_eq!(item.unwrap().filter_type, FilterType::Trash);
    }

    #[test]
    fn test_focus_state() {
        let mut panel = FilterPanel::new();
        assert!(!panel.focused);

        panel.on_focus_gain().unwrap();
        assert!(panel.focused);

        panel.on_focus_loss().unwrap();
        assert!(!panel.focused);
    }

    #[test]
    fn test_custom_items() {
        let items = vec![
            FilterItem::new(FilterType::Favorite, "Favorites", "★"),
        ];
        let panel = FilterPanel::with_items(items);
        assert_eq!(panel.items.len(), 1);
        assert_eq!(panel.items[0].label, "Favorites");
    }
}
