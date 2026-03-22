//! Group Picker Popup Component
//!
//! A modal popup for selecting a group to move a password into.

use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

/// Group picker popup
pub struct GroupPicker {
    id: ComponentId,
    /// List of (group_id, group_name) — None id means "Ungrouped"
    groups: Vec<(Option<String>, String)>,
    /// Currently highlighted index
    selected: usize,
    /// The password being moved
    password_id: String,
    /// Whether the picker is visible
    visible: bool,
}

impl GroupPicker {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(200),
            groups: Vec::new(),
            selected: 0,
            password_id: String::new(),
            visible: false,
        }
    }

    /// Show the picker for a specific password
    pub fn show(&mut self, password_id: String, groups: Vec<(Option<String>, String)>) {
        self.password_id = password_id;
        self.groups = groups;
        self.selected = 0;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the selected group_id (None = ungrouped)
    pub fn selected_group_id(&self) -> Option<&str> {
        self.groups
            .get(self.selected)
            .and_then(|(id, _)| id.as_deref())
    }

    pub fn password_id(&self) -> &str {
        &self.password_id
    }

    /// Render the picker popup using frame (for overlay)
    pub fn render_frame(&self, frame: &mut ratatui::Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let w = 40u16.min(area.width);
        let h = ((self.groups.len() as u16).saturating_add(4)).min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(w)) / 2;
        let y = (area.height.saturating_sub(h)) / 2;
        let popup_area = Rect::new(x, y, w, h);

        // Clear background
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Move to Group ");

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Render group list
        let mut lines = Vec::new();
        for (i, (_id, name)) in self.groups.iter().enumerate() {
            let style = if i == self.selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Rgb(40, 60, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(200, 200, 215))
            };
            lines.push(Line::from(Span::styled(format!("  {}  ", name), style)));
        }

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

impl Interactive for GroupPicker {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release || !self.visible {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                self.hide();
                HandleResult::Consumed
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.groups.len() {
                    self.selected += 1;
                }
                HandleResult::Consumed
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                HandleResult::Consumed // Caller reads selected_group_id()
            }
            _ => HandleResult::Consumed, // Absorb all keys when visible
        }
    }
}

impl Component for GroupPicker {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

impl Render for GroupPicker {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Simplified buffer-based render for trait compliance
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Move to Group ");
        block.render(area, buf);
    }
}

impl Default for GroupPicker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_picker_new() {
        let picker = GroupPicker::new();
        assert!(!picker.is_visible());
        assert_eq!(picker.password_id(), "");
        assert!(picker.selected_group_id().is_none());
    }

    #[test]
    fn test_group_picker_show_hide() {
        let mut picker = GroupPicker::new();
        let groups = vec![
            (None, "Ungrouped".to_string()),
            (Some("g1".to_string()), "Work".to_string()),
        ];
        picker.show("pw-123".to_string(), groups);

        assert!(picker.is_visible());
        assert_eq!(picker.password_id(), "pw-123");
        // First item is Ungrouped (None id)
        assert!(picker.selected_group_id().is_none());

        picker.hide();
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_group_picker_navigation() {
        let mut picker = GroupPicker::new();
        let groups = vec![
            (None, "Ungrouped".to_string()),
            (Some("g1".to_string()), "Work".to_string()),
            (Some("g2".to_string()), "Personal".to_string()),
        ];
        picker.show("pw-1".to_string(), groups);

        let down = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty());
        let up = KeyEvent::new(KeyCode::Up, crossterm::event::KeyModifiers::empty());

        // Navigate down
        picker.handle_key(down);
        assert_eq!(picker.selected_group_id(), Some("g1"));

        picker.handle_key(down);
        assert_eq!(picker.selected_group_id(), Some("g2"));

        // Can't go past end
        picker.handle_key(down);
        assert_eq!(picker.selected_group_id(), Some("g2"));

        // Navigate up
        picker.handle_key(up);
        assert_eq!(picker.selected_group_id(), Some("g1"));

        picker.handle_key(up);
        assert!(picker.selected_group_id().is_none()); // back to Ungrouped

        // Can't go above start
        picker.handle_key(up);
        assert!(picker.selected_group_id().is_none());
    }

    #[test]
    fn test_group_picker_esc_hides() {
        let mut picker = GroupPicker::new();
        picker.show("pw-1".to_string(), vec![(None, "Ungrouped".to_string())]);
        assert!(picker.is_visible());

        let esc = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::empty());
        let result = picker.handle_key(esc);
        assert!(matches!(result, HandleResult::Consumed));
        assert!(!picker.is_visible());
    }

    #[test]
    fn test_group_picker_ignores_keys_when_hidden() {
        let mut picker = GroupPicker::new();
        let down = KeyEvent::new(KeyCode::Down, crossterm::event::KeyModifiers::empty());
        let result = picker.handle_key(down);
        assert!(matches!(result, HandleResult::Ignored));
    }
}
