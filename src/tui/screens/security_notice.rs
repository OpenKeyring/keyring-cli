//! Security Notice Screen
//!
//! Critical warning about master password recovery

use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

/// Security notice screen
pub struct SecurityNoticeScreen {
    /// User acknowledged the warning
    acknowledged: bool,
    /// Component ID
    id: ComponentId,
}

impl SecurityNoticeScreen {
    /// Create new security notice screen
    pub fn new() -> Self {
        Self {
            acknowledged: false,
            id: ComponentId::new(3011),
        }
    }

    /// Check if user acknowledged
    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged
    }

    /// Set acknowledgment state
    pub fn set_acknowledged(&mut self, value: bool) {
        self.acknowledged = value;
    }
}

impl Default for SecurityNoticeScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for SecurityNoticeScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("⚠️  Important Security Notice")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        let notice_lines = vec![
            Line::from(""),
            Line::from("┌─────────────────────────────────────────────────────────────┐")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│                                                             │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│   ⚠️  YOUR MASTER PASSWORD CANNOT BE RECOVERED!            │")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Line::from("│                                                             │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│   If you forget your master password, you will lose        │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│   access to ALL your passwords permanently.                │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│                                                             │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│   We STRONGLY recommend setting up a PassKey               │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│   (24-word recovery phrase) as a backup.                   │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("│                                                             │")
                .style(Style::default().fg(Color::Yellow)),
            Line::from("└─────────────────────────────────────────────────────────────┘")
                .style(Style::default().fg(Color::Yellow)),
            Line::from(""),
            Line::from(""),
            Line::from({
                if self.acknowledged {
                    "[✓] I understand the risk and want to continue".to_string()
                } else {
                    "[ ] I understand the risk and want to continue".to_string()
                }
            })
            .style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from("Press SPACE to acknowledge, then ENTER to continue")
                .style(Style::default().fg(Color::DarkGray)),
        ];

        let paragraph = Paragraph::new(notice_lines).alignment(Alignment::Center);

        paragraph.render(inner, buf);
    }
}

impl Interactive for SecurityNoticeScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Char(' ') => {
                self.acknowledged = !self.acknowledged;
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.acknowledged {
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for SecurityNoticeScreen {
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
        let screen = SecurityNoticeScreen::new();
        assert!(!screen.is_acknowledged());
    }

    #[test]
    fn test_toggle_acknowledgment() {
        let mut screen = SecurityNoticeScreen::new();

        // Press space to acknowledge
        let result = screen.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert!(screen.is_acknowledged());

        // Press space again to unacknowledge
        let result = screen.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(matches!(result, HandleResult::NeedsRender));
        assert!(!screen.is_acknowledged());
    }

    #[test]
    fn test_enter_without_acknowledgment() {
        let mut screen = SecurityNoticeScreen::new();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Ignored));
    }

    #[test]
    fn test_enter_with_acknowledgment() {
        let mut screen = SecurityNoticeScreen::new();
        screen.set_acknowledged(true);
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }

    #[test]
    fn test_set_acknowledged() {
        let mut screen = SecurityNoticeScreen::new();
        screen.set_acknowledged(true);
        assert!(screen.is_acknowledged());
        screen.set_acknowledged(false);
        assert!(!screen.is_acknowledged());
    }
}
