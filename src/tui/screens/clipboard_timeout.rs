//! Clipboard Timeout Configuration Screen

use crate::tui::screens::wizard::ClipboardTimeout;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Clipboard timeout configuration screen
pub struct ClipboardTimeoutScreen {
    timeout: ClipboardTimeout,
    id: ComponentId,
}

impl ClipboardTimeoutScreen {
    /// Create new screen with default timeout
    pub fn new() -> Self {
        Self {
            timeout: ClipboardTimeout::default(),
            id: ComponentId::new(3014),
        }
    }

    /// Create with specific timeout
    pub fn with_timeout(timeout: ClipboardTimeout) -> Self {
        Self {
            timeout,
            id: ComponentId::new(3014),
        }
    }

    /// Get the selected timeout
    pub fn timeout(&self) -> ClipboardTimeout {
        self.timeout
    }

    /// Set the timeout
    pub fn set_timeout(&mut self, timeout: ClipboardTimeout) {
        self.timeout = timeout;
    }
}

impl Default for ClipboardTimeoutScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ClipboardTimeoutScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("📋 Clipboard Auto-Clear Timeout")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let options = [
            (ClipboardTimeout::Seconds10, "10 seconds"),
            (ClipboardTimeout::Seconds30, "30 seconds (recommended)"),
            (ClipboardTimeout::Seconds60, "60 seconds"),
        ];

        let mut lines = vec![
            Line::from(""),
            Line::from("When you copy a password, it will be automatically"),
            Line::from("cleared from your clipboard after this time:"),
            Line::from(""),
        ];

        for (timeout, label) in &options {
            let is_selected = *timeout == self.timeout;
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
            Line::from("  [↑↓] Select   [Enter] Continue")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        paragraph.render(inner, buf);
    }
}

impl Interactive for ClipboardTimeoutScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Up => {
                self.timeout = match self.timeout {
                    ClipboardTimeout::Seconds10 => ClipboardTimeout::Seconds60,
                    ClipboardTimeout::Seconds30 => ClipboardTimeout::Seconds10,
                    ClipboardTimeout::Seconds60 => ClipboardTimeout::Seconds30,
                };
                HandleResult::NeedsRender
            }
            KeyCode::Down => {
                self.timeout = match self.timeout {
                    ClipboardTimeout::Seconds10 => ClipboardTimeout::Seconds30,
                    ClipboardTimeout::Seconds30 => ClipboardTimeout::Seconds60,
                    ClipboardTimeout::Seconds60 => ClipboardTimeout::Seconds10,
                };
                HandleResult::NeedsRender
            }
            KeyCode::Enter => HandleResult::Consumed,
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for ClipboardTimeoutScreen {
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
        let screen = ClipboardTimeoutScreen::new();
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds30); // Default
    }

    #[test]
    fn test_with_timeout() {
        let screen = ClipboardTimeoutScreen::with_timeout(ClipboardTimeout::Seconds60);
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds60);
    }

    #[test]
    fn test_set_timeout() {
        let mut screen = ClipboardTimeoutScreen::new();
        screen.set_timeout(ClipboardTimeout::Seconds10);
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds10);
    }

    #[test]
    fn test_navigation_down() {
        let mut screen = ClipboardTimeoutScreen::new();
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds30);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds60);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds10);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds30);
    }

    #[test]
    fn test_navigation_up() {
        let mut screen = ClipboardTimeoutScreen::new();
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds30);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds10);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds60);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.timeout(), ClipboardTimeout::Seconds30);
    }

    #[test]
    fn test_enter_confirms() {
        let mut screen = ClipboardTimeoutScreen::new();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }

    #[test]
    fn test_timeout_seconds() {
        assert_eq!(ClipboardTimeout::Seconds10.seconds(), 10);
        assert_eq!(ClipboardTimeout::Seconds30.seconds(), 30);
        assert_eq!(ClipboardTimeout::Seconds60.seconds(), 60);
    }
}
