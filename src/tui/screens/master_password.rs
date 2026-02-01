//! Master Password Setup Screen
//!
//! Allows users to set up their device-specific master password for encrypting the Passkey.

use crate::health::strength::calculate_strength;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Password strength indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    /// Weak password
    Weak,
    /// Medium password
    Medium,
    /// Strong password
    Strong,
}

impl PasswordStrength {
    /// Get display text for this strength level
    pub fn display(&self) -> &str {
        match self {
            PasswordStrength::Weak => "弱",
            PasswordStrength::Medium => "中",
            PasswordStrength::Strong => "强",
        }
    }

    /// Get color for this strength level
    pub fn color(&self) -> Color {
        match self {
            PasswordStrength::Weak => Color::Red,
            PasswordStrength::Medium => Color::Yellow,
            PasswordStrength::Strong => Color::Green,
        }
    }

    /// Get icon for this strength level
    pub fn icon(&self) -> &str {
        match self {
            PasswordStrength::Weak => "⚠️",
            PasswordStrength::Medium => "🔒",
            PasswordStrength::Strong => "🔐",
        }
    }
}

/// Master password setup screen
#[derive(Debug, Clone)]
pub struct MasterPasswordScreen {
    /// First password input
    password_input: String,
    /// Confirmation password input
    confirm_input: String,
    /// Whether showing first password field (true) or confirmation (false)
    show_first: bool,
    /// Current password strength
    strength: PasswordStrength,
    /// Validation error message
    validation_error: Option<String>,
    /// Whether passwords match
    passwords_match: bool,
}

impl MasterPasswordScreen {
    /// Create a new master password screen
    pub fn new() -> Self {
        Self {
            password_input: String::new(),
            confirm_input: String::new(),
            show_first: true,
            strength: PasswordStrength::Weak,
            validation_error: None,
            passwords_match: false,
        }
    }

    /// Get current password input
    pub fn password_input(&self) -> &str {
        &self.password_input
    }

    /// Get confirmation input
    pub fn confirm_input(&self) -> &str {
        &self.confirm_input
    }

    /// Check if showing first password field
    pub fn is_showing_first(&self) -> bool {
        self.show_first
    }

    /// Get password strength
    pub fn strength(&self) -> PasswordStrength {
        self.strength
    }

    /// Get validation error
    pub fn validation_error(&self) -> Option<&str> {
        self.validation_error.as_deref()
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }

        if self.show_first {
            self.password_input.push(c);
            self.update_strength();
            self.validation_error = None;
        } else {
            self.confirm_input.push(c);
            self.update_match_status();
            self.validation_error = None;
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.show_first {
            self.password_input.pop();
            self.update_strength();
        } else {
            self.confirm_input.pop();
            self.update_match_status();
        }
        self.validation_error = None;
    }

    /// Move to confirmation field
    pub fn next(&mut self) {
        if self.show_first && !self.password_input.is_empty() {
            self.show_first = false;
        }
    }

    /// Go back to password field
    pub fn back(&mut self) {
        if !self.show_first {
            self.show_first = true;
        }
    }

    /// Check if the wizard can complete
    pub fn can_complete(&self) -> bool {
        !self.password_input.is_empty()
            && !self.confirm_input.is_empty()
            && self.passwords_match
            && self.password_input.len() >= 8
    }

    /// Get the password if valid
    pub fn get_password(&self) -> Option<String> {
        if self.can_complete() {
            Some(self.password_input.clone())
        } else {
            None
        }
    }

    /// Update password strength based on current input
    fn update_strength(&mut self) {
        let score = calculate_strength(&self.password_input);
        self.strength = if score < 50 {
            PasswordStrength::Weak
        } else if score < 70 {
            PasswordStrength::Medium
        } else {
            PasswordStrength::Strong
        };
    }

    /// Update match status
    fn update_match_status(&mut self) {
        self.passwords_match =
            !self.confirm_input.is_empty() && self.password_input == self.confirm_input;
    }

    /// Validate and return error if any
    pub fn validate(&self) -> Result<(), String> {
        if self.password_input.is_empty() {
            return Err("请输入主密码".to_string());
        }

        if self.password_input.len() < 8 {
            return Err("主密码至少需要 8 个字符".to_string());
        }

        if self.confirm_input.is_empty() {
            return Err("请再次输入主密码".to_string());
        }

        if !self.passwords_match {
            return Err("两次输入的密码不匹配".to_string());
        }

        Ok(())
    }

    /// Clear all inputs
    pub fn clear(&mut self) {
        self.password_input.clear();
        self.confirm_input.clear();
        self.show_first = true;
        self.strength = PasswordStrength::Weak;
        self.validation_error = None;
        self.passwords_match = false;
    }

    /// Render the master password screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(5), // Password input
                    Constraint::Length(5), // Confirm input
                    Constraint::Length(2), // Status/Error
                    Constraint::Min(0),    // Spacer
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "设置本设备的主密码",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        // Password input
        let password_display = "•".repeat(self.password_input.len());
        let password_field = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "主密码: ",
                    Style::default().fg(if self.show_first {
                        Color::Cyan
                    } else {
                        Color::Gray
                    }),
                ),
                Span::styled(
                    if password_display.is_empty() {
                        if self.show_first {
                            "在此输入..."
                        } else {
                            ""
                        }
                    } else {
                        password_display.as_str()
                    },
                    Style::default().fg(if self.show_first {
                        Color::White
                    } else {
                        Color::Gray
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{} 强度: {}", self.strength.icon(), self.strength.display()),
                    Style::default()
                        .fg(self.strength.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ("),
                Span::styled(
                    format!("{}", calculate_strength(&self.password_input)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("/100)"),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if self.show_first {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .title(" 主密码 "),
        )
        .wrap(Wrap { trim: false });

        frame.render_widget(password_field, chunks[2]);

        // Confirm input
        let confirm_display = "•".repeat(self.confirm_input.len());
        let confirm_field = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "确认密码: ",
                Style::default().fg(if !self.show_first {
                    Color::Cyan
                } else {
                    Color::Gray
                }),
            ),
            Span::styled(
                if confirm_display.is_empty() {
                    if !self.show_first {
                        "在此输入..."
                    } else {
                        ""
                    }
                } else {
                    confirm_display.as_str()
                },
                Style::default().fg(if !self.show_first {
                    Color::White
                } else {
                    Color::Gray
                }),
            ),
            Span::raw(if !self.confirm_input.is_empty() && self.passwords_match {
                " ✓"
            } else if !self.confirm_input.is_empty() {
                " ✗"
            } else {
                ""
            }),
            Span::styled(
                if !self.confirm_input.is_empty() && self.passwords_match {
                    " 匹配"
                } else if !self.confirm_input.is_empty() {
                    " 不匹配"
                } else {
                    ""
                },
                Style::default().fg(if self.passwords_match {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ])])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if !self.show_first {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .title(" 确认密码 "),
        )
        .wrap(Wrap { trim: false });

        frame.render_widget(confirm_field, chunks[3]);

        // Status/Error
        let status = if let Some(error) = &self.validation_error {
            Paragraph::new(Line::from(vec![
                Span::styled("✗ ", Style::default().fg(Color::Red)),
                Span::styled(error, Style::default().fg(Color::Red)),
            ]))
        } else if self.can_complete() {
            Paragraph::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::styled("密码设置完成", Style::default().fg(Color::Green)),
            ]))
        } else if self.show_first {
            Paragraph::new(Line::from(Span::styled(
                "提示: 密码至少需要 8 个字符",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))
        } else {
            Paragraph::new(Line::from(""))
        };

        frame.render_widget(status, chunks[4]);

        // Info hint
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "此密码仅用于加密 Passkey",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    "与其他设备的密码可以不同",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]),
        ])
        .wrap(Wrap { trim: true });

        frame.render_widget(hint, chunks[5]);

        // Footer
        let footer_spans = vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if self.can_complete() {
                ": 完成    "
            } else if self.show_first && !self.password_input.is_empty() {
                ": 继续    "
            } else {
                "         "
            }),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": 切换    "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": 返回"),
        ];

        let footer = Paragraph::new(Line::from(footer_spans))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[6]);
    }
}

impl Default for MasterPasswordScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_password_new() {
        let screen = MasterPasswordScreen::new();
        assert!(screen.is_showing_first());
        assert_eq!(screen.password_input(), "");
        assert_eq!(screen.confirm_input(), "");
    }

    #[test]
    fn test_master_password_handle_char() {
        let mut screen = MasterPasswordScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_char('c');
        assert_eq!(screen.password_input(), "abc");
    }

    #[test]
    fn test_master_password_handle_backspace() {
        let mut screen = MasterPasswordScreen::new();
        screen.handle_char('a');
        screen.handle_char('b');
        screen.handle_backspace();
        assert_eq!(screen.password_input(), "a");
    }

    #[test]
    fn test_master_password_next() {
        let mut screen = MasterPasswordScreen::new();
        screen.handle_char('a');
        screen.next();
        assert!(!screen.is_showing_first());
    }

    #[test]
    fn test_master_password_back() {
        let mut screen = MasterPasswordScreen::new();
        screen.handle_char('a');
        screen.next();
        screen.back();
        assert!(screen.is_showing_first());
    }

    #[test]
    fn test_master_password_can_complete() {
        let mut screen = MasterPasswordScreen::new();
        assert!(!screen.can_complete());

        screen.password_input = "short".to_string();
        screen.confirm_input = "short".to_string();
        screen.update_match_status();
        assert!(!screen.can_complete()); // Too short

        screen.password_input = "longenough".to_string();
        screen.confirm_input = "longenough".to_string();
        screen.update_match_status();
        assert!(screen.can_complete());

        screen.confirm_input = "different".to_string();
        screen.update_match_status();
        assert!(!screen.can_complete()); // Don't match
    }

    #[test]
    fn test_master_password_validate() {
        let mut screen = MasterPasswordScreen::new();

        assert!(screen.validate().is_err()); // Empty

        screen.password_input = "short".to_string();
        assert!(screen.validate().is_err()); // Too short

        screen.password_input = "longenough".to_string();
        assert!(screen.validate().is_err()); // No confirmation

        screen.confirm_input = "different".to_string();
        screen.update_match_status();
        assert!(screen.validate().is_err()); // Don't match

        screen.confirm_input = "longenough".to_string();
        screen.update_match_status();
        assert!(screen.validate().is_ok()); // Valid
    }

    #[test]
    fn test_password_strength_display() {
        assert_eq!(PasswordStrength::Weak.display(), "弱");
        assert_eq!(PasswordStrength::Medium.display(), "中");
        assert_eq!(PasswordStrength::Strong.display(), "强");
    }

    #[test]
    fn test_password_strength_color() {
        assert_eq!(PasswordStrength::Weak.color(), Color::Red);
        assert_eq!(PasswordStrength::Medium.color(), Color::Yellow);
        assert_eq!(PasswordStrength::Strong.color(), Color::Green);
    }
}
