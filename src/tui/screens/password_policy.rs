//! Password Policy Configuration Screen
//!
//! Configure default password generation settings

use crate::tui::screens::wizard::{PasswordPolicyConfig, PasswordType};
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Password policy configuration screen
pub struct PasswordPolicyScreen {
    /// Current config
    config: PasswordPolicyConfig,
    /// Currently focused field
    focused: usize,
    /// Component ID
    id: ComponentId,
}

impl PasswordPolicyScreen {
    /// Create new screen with default config
    pub fn new() -> Self {
        Self {
            config: PasswordPolicyConfig::default(),
            focused: 0,
            id: ComponentId::new(3013),
        }
    }

    /// Create with existing config
    pub fn with_config(config: PasswordPolicyConfig) -> Self {
        Self {
            config,
            focused: 0,
            id: ComponentId::new(3013),
        }
    }

    /// Get the configured policy
    pub fn config(&self) -> PasswordPolicyConfig {
        self.config
    }

    /// Set the config
    pub fn set_config(&mut self, config: PasswordPolicyConfig) {
        self.config = config;
    }

    fn type_name(t: PasswordType) -> &'static str {
        match t {
            PasswordType::Random => "Random Password",
            PasswordType::Memorable => "Memorable (Word-based)",
            PasswordType::Pin => "PIN Code",
        }
    }

    /// Get currently focused field index
    pub fn focused(&self) -> usize {
        self.focused
    }

    /// Set focused field
    pub fn set_focused(&mut self, idx: usize) {
        self.focused = idx.min(3);
    }
}

impl Default for PasswordPolicyScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for PasswordPolicyScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::Widget;

        let block = Block::default()
            .title("🔑 Password Generation Policy")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let fields = [
            ("Password Type", Self::type_name(self.config.default_type)),
            ("Default Length", &format!("{}", self.config.default_length)),
            ("Min Digits", &format!("{}", self.config.min_digits)),
            ("Min Special Chars", &format!("{}", self.config.min_special)),
        ];

        let mut lines = vec![
            Line::from(""),
            Line::from("Configure your default password generation settings:"),
            Line::from(""),
        ];

        for (i, (label, value)) in fields.iter().enumerate() {
            let is_focused = i == self.focused;
            let style = if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw(format!("  {}: ", label)),
                Span::styled(format!("[{}]", value), style),
                Span::raw(if is_focused { " ◀" } else { "" }),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("  Tip: You can customize these settings later in the app")
                .style(Style::default().fg(Color::DarkGray)),
        );
        lines.push(Line::from(""));
        lines.push(
            Line::from("  [↑↓] Navigate   [←→] Change value   [Enter] Continue")
                .style(Style::default().fg(Color::DarkGray)),
        );

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

        paragraph.render(inner, buf);
    }
}

impl Interactive for PasswordPolicyScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Up => {
                if self.focused > 0 {
                    self.focused -= 1;
                }
                HandleResult::NeedsRender
            }
            KeyCode::Down => {
                if self.focused < 3 {
                    self.focused += 1;
                }
                HandleResult::NeedsRender
            }
            KeyCode::Left | KeyCode::Right => {
                let is_left = key.code == KeyCode::Left;
                match self.focused {
                    0 => {
                        // Password type - cycle through options
                        self.config.default_type = match self.config.default_type {
                            PasswordType::Random => {
                                if is_left {
                                    PasswordType::Pin
                                } else {
                                    PasswordType::Memorable
                                }
                            }
                            PasswordType::Memorable => {
                                if is_left {
                                    PasswordType::Random
                                } else {
                                    PasswordType::Pin
                                }
                            }
                            PasswordType::Pin => {
                                if is_left {
                                    PasswordType::Memorable
                                } else {
                                    PasswordType::Random
                                }
                            }
                        };
                    }
                    1 => {
                        // Length: 8-64
                        if is_left && self.config.default_length > 8 {
                            self.config.default_length -= 1;
                        } else if !is_left && self.config.default_length < 64 {
                            self.config.default_length += 1;
                        }
                    }
                    2 => {
                        // Min digits: 0-10
                        if is_left && self.config.min_digits > 0 {
                            self.config.min_digits -= 1;
                        } else if !is_left && self.config.min_digits < 10 {
                            self.config.min_digits += 1;
                        }
                    }
                    3 => {
                        // Min special: 0-10
                        if is_left && self.config.min_special > 0 {
                            self.config.min_special -= 1;
                        } else if !is_left && self.config.min_special < 10 {
                            self.config.min_special += 1;
                        }
                    }
                    _ => {}
                }
                HandleResult::NeedsRender
            }
            KeyCode::Enter => HandleResult::Consumed,
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for PasswordPolicyScreen {
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
        let screen = PasswordPolicyScreen::new();
        assert_eq!(screen.focused(), 0);

        let config = screen.config();
        assert_eq!(config.default_type, PasswordType::Random);
        assert_eq!(config.default_length, 16);
        assert_eq!(config.min_digits, 2);
        assert_eq!(config.min_special, 1);
    }

    #[test]
    fn test_with_config() {
        let config = PasswordPolicyConfig {
            default_type: PasswordType::Memorable,
            default_length: 24,
            min_digits: 4,
            min_special: 2,
        };
        let screen = PasswordPolicyScreen::with_config(config);

        let result = screen.config();
        assert_eq!(result.default_type, PasswordType::Memorable);
        assert_eq!(result.default_length, 24);
        assert_eq!(result.min_digits, 4);
        assert_eq!(result.min_special, 2);
    }

    #[test]
    fn test_navigation_up_down() {
        let mut screen = PasswordPolicyScreen::new();

        assert_eq!(screen.focused(), 0);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.focused(), 1);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.focused(), 2);

        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.focused(), 3);

        // Should not go beyond 3
        screen.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(screen.focused(), 3);

        // Go back up
        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.focused(), 2);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.focused(), 1);

        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.focused(), 0);

        // Should not go below 0
        screen.handle_key(KeyEvent::from(KeyCode::Up));
        assert_eq!(screen.focused(), 0);
    }

    #[test]
    fn test_change_password_type() {
        let mut screen = PasswordPolicyScreen::new();
        assert_eq!(screen.config().default_type, PasswordType::Random);

        // Right to cycle forward
        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().default_type, PasswordType::Memorable);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().default_type, PasswordType::Pin);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().default_type, PasswordType::Random);

        // Left to cycle backward
        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().default_type, PasswordType::Pin);

        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().default_type, PasswordType::Memorable);
    }

    #[test]
    fn test_change_length() {
        let mut screen = PasswordPolicyScreen::new();
        screen.set_focused(1);

        assert_eq!(screen.config().default_length, 16);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().default_length, 17);

        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().default_length, 16);

        // Test bounds
        screen.set_config(PasswordPolicyConfig {
            default_length: 8,
            ..Default::default()
        });
        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().default_length, 8); // Min

        screen.set_config(PasswordPolicyConfig {
            default_length: 64,
            ..Default::default()
        });
        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().default_length, 64); // Max
    }

    #[test]
    fn test_change_min_digits() {
        let mut screen = PasswordPolicyScreen::new();
        screen.set_focused(2);

        assert_eq!(screen.config().min_digits, 2);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().min_digits, 3);

        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().min_digits, 2);
    }

    #[test]
    fn test_change_min_special() {
        let mut screen = PasswordPolicyScreen::new();
        screen.set_focused(3);

        assert_eq!(screen.config().min_special, 1);

        screen.handle_key(KeyEvent::from(KeyCode::Right));
        assert_eq!(screen.config().min_special, 2);

        screen.handle_key(KeyEvent::from(KeyCode::Left));
        assert_eq!(screen.config().min_special, 1);
    }

    #[test]
    fn test_enter_confirms() {
        let mut screen = PasswordPolicyScreen::new();
        let result = screen.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, HandleResult::Consumed));
    }
}
