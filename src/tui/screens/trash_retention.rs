//! Trash Retention Configuration Screen

use crate::tui::screens::wizard::TrashRetention;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Trash retention configuration screen
pub struct TrashRetentionScreen {
    retention: TrashRetention,
    id: ComponentId,
}

impl TrashRetentionScreen {
    /// Create new screen with default retention
    pub fn new() -> Self {
        Self {
            retention: TrashRetention::default(),
            id: ComponentId::new(3015),
        }
    }

    /// Create with specific retention
    pub fn with_retention(retention: TrashRetention) -> Self {
        Self {
            retention,
            id: ComponentId::new(3015),
        }
    }

    /// Get the selected retention
    pub fn retention(&self) -> TrashRetention {
        self.retention
    }

    /// Set the retention
    pub fn set_retention(&mut self, retention: TrashRetention) {
        self.retention = retention;
    }
}

impl Default for TrashRetentionScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for TrashRetentionScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("🗑️  Trash Retention Period")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let options = [
            (TrashRetention::Days7, "7 days"),
            (TrashRetention::Days30, "30 days (recommended)"),
            (TrashRetention::Days90, "90 days"),
        ];

        let mut lines = vec![
            Line::from(""),
            Line::from("Deleted passwords will be kept in trash for:"),
            Line::from(""),
        ];

        for (retention, label) in &options {
            let is_selected = *retention == self.retention;
            let marker = if is_selected { "●" } else { "○" };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(format!("  {} ", marker)),
                Span::styled(*label, style),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("  After this period, deleted passwords will be permanently removed.")
                .style(Style::default().fg(Color::DarkGray)),
        );
        lines.push(Line::from(""));
        lines.push(
            Line::from("  [↑↓] Select   [Enter] Continue")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        paragraph.render(inner, buf);
    }
}

impl Interactive for TrashRetentionScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Up => {
                self.retention = match self.retention {
                    TrashRetention::Days7 => TrashRetention::Days90,
                    TrashRetention::Days30 => TrashRetention::Days7,
                    TrashRetention::Days90 => TrashRetention::Days30,
                };
                HandleResult::NeedsRender
            }
            KeyCode::Down => {
                self.retention = match self.retention {
                    TrashRetention::Days7 => TrashRetention::Days30,
                    TrashRetention::Days30 => TrashRetention::Days90,
                    TrashRetention::Days90 => TrashRetention::Days7,
                };
                HandleResult::NeedsRender
            }
            KeyCode::Enter => HandleResult::Consumed,
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for TrashRetentionScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_screen() {
        let screen = TrashRetentionScreen::new();
        assert_eq!(screen.retention(), TrashRetention::Days30); // Default
    }

    #[test]
    fn test_with_retention() {
        let screen = TrashRetentionScreen::with_retention(TrashRetention::Days90);
        assert_eq!(screen.retention(), TrashRetention::Days90);
    }

    #[test]
    fn test_set_retention() {
        let mut screen = TrashRetentionScreen::new();
        screen.set_retention(TrashRetention::Days7);
        assert_eq!(screen.retention(), TrashRetention::Days7);
    }

    #[test]
    fn test_navigation_down() {
        let mut screen = TrashRetentionScreen::new();
        assert_eq!(screen.retention(), TrashRetention::Days30);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.retention(), TrashRetention::Days90);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.retention(), TrashRetention::Days7);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.retention(), TrashRetention::Days30);
    }

    #[test]
    fn test_navigation_up() {
        let mut screen = TrashRetentionScreen::new();
        assert_eq!(screen.retention(), TrashRetention::Days30);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.retention(), TrashRetention::Days7);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.retention(), TrashRetention::Days90);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.retention(), TrashRetention::Days30);
    }

    #[test]
    fn test_enter_confirms() {
        let mut screen = TrashRetentionScreen::new();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }

    #[test]
    fn test_retention_days() {
        assert_eq!(TrashRetention::Days7.days(), 7);
        assert_eq!(TrashRetention::Days30.days(), 30);
        assert_eq!(TrashRetention::Days90.days(), 90);
    }
}
