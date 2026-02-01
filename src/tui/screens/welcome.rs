//! Welcome Screen for Onboarding Wizard
//!
//! First screen of the onboarding wizard, allowing users to choose between
//! generating a new Passkey or importing an existing one.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// User's choice for Passkey setup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeChoice {
    /// Generate a new 24-word Passkey
    GenerateNew,
    /// Import an existing Passkey
    ImportExisting,
}

impl WelcomeChoice {
    /// Get display text for this choice
    pub fn display_text(&self) -> &str {
        match self {
            WelcomeChoice::GenerateNew => "全新使用（生成新的 Passkey）",
            WelcomeChoice::ImportExisting => "导入已有 Passkey",
        }
    }

    /// Get description text for this choice
    pub fn description(&self) -> &str {
        match self {
            WelcomeChoice::GenerateNew => "将生成一个 24 词的 Passkey",
            WelcomeChoice::ImportExisting => "如果您已经在其他设备上使用过",
        }
    }

    /// Toggle between choices
    pub fn toggle(&self) -> Self {
        match self {
            WelcomeChoice::GenerateNew => WelcomeChoice::ImportExisting,
            WelcomeChoice::ImportExisting => WelcomeChoice::GenerateNew,
        }
    }
}

/// Welcome screen for the onboarding wizard
#[derive(Debug, Clone)]
pub struct WelcomeScreen {
    /// Currently selected choice
    selected: WelcomeChoice,
}

impl WelcomeScreen {
    /// Create a new welcome screen
    pub fn new() -> Self {
        Self {
            selected: WelcomeChoice::GenerateNew,
        }
    }

    /// Get the current selected choice
    pub fn selected(&self) -> WelcomeChoice {
        self.selected
    }

    /// Toggle between GenerateNew and ImportExisting
    pub fn toggle(&mut self) {
        self.selected = self.selected.toggle();
    }

    /// Set the choice directly
    pub fn set_choice(&mut self, choice: WelcomeChoice) {
        self.selected = choice;
    }

    /// Render the welcome screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Length(2), // Spacer
                    Constraint::Length(2), // Welcome message
                    Constraint::Length(2), // Spacer
                    Constraint::Length(2), // Prompt
                    Constraint::Min(0),    // Choices
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        // Title
        let title = Paragraph::new(vec![Line::from(Span::styled(
            "OpenKeyring 初始化向导",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(title, chunks[0]);

        // Welcome message
        let welcome = Paragraph::new(vec![Line::from(Span::styled(
            "欢迎使用 OpenKeyring！",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Center);

        frame.render_widget(welcome, chunks[2]);

        // Prompt
        let prompt = Paragraph::new(vec![Line::from(Span::styled(
            "选择设置方式:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))])
        .alignment(Alignment::Left);

        frame.render_widget(prompt, chunks[4]);

        // Choices
        let choices = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if self.selected == WelcomeChoice::GenerateNew {
                        "●"
                    } else {
                        "○"
                    },
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    WelcomeChoice::GenerateNew.display_text(),
                    Style::default()
                        .fg(if self.selected == WelcomeChoice::GenerateNew {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    WelcomeChoice::GenerateNew.description(),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    if self.selected == WelcomeChoice::ImportExisting {
                        "●"
                    } else {
                        "○"
                    },
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    WelcomeChoice::ImportExisting.display_text(),
                    Style::default()
                        .fg(if self.selected == WelcomeChoice::ImportExisting {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    WelcomeChoice::ImportExisting.description(),
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 选项 / Options "),
        )
        .wrap(Wrap { trim: false });

        frame.render_widget(choices, chunks[5]);

        // Footer with keyboard hints
        let footer = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": 下一步    "),
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": 选择    "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": 退出"),
        ])])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[6]);
    }
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_choice_toggle() {
        assert_eq!(
            WelcomeChoice::GenerateNew.toggle(),
            WelcomeChoice::ImportExisting
        );
        assert_eq!(
            WelcomeChoice::ImportExisting.toggle(),
            WelcomeChoice::GenerateNew
        );
    }

    #[test]
    fn test_welcome_screen_new() {
        let screen = WelcomeScreen::new();
        assert_eq!(screen.selected(), WelcomeChoice::GenerateNew);
    }

    #[test]
    fn test_welcome_screen_toggle() {
        let mut screen = WelcomeScreen::new();
        screen.toggle();
        assert_eq!(screen.selected(), WelcomeChoice::ImportExisting);
        screen.toggle();
        assert_eq!(screen.selected(), WelcomeChoice::GenerateNew);
    }

    #[test]
    fn test_welcome_screen_set_choice() {
        let mut screen = WelcomeScreen::new();
        screen.set_choice(WelcomeChoice::ImportExisting);
        assert_eq!(screen.selected(), WelcomeChoice::ImportExisting);
    }
}
