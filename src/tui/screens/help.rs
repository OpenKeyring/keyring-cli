//! Help Screen
//!
//! TUI screen for displaying keyboard shortcuts and help information.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// A keyboard shortcut entry
#[derive(Debug, Clone)]
pub struct Shortcut {
    /// Key combination (e.g., "Ctrl+Q" or "↑")
    pub keys: String,
    /// Action description (e.g., "Quit")
    pub action: String,
}

/// A help section containing related shortcuts
#[derive(Debug, Clone)]
pub struct HelpSection {
    /// Section title
    pub title: String,
    /// Shortcuts in this section
    pub shortcuts: Vec<Shortcut>,
}

/// Help screen
#[derive(Debug, Clone)]
pub struct HelpScreen {
    /// Help sections
    sections: Vec<HelpSection>,
    /// Current scroll position (line number)
    scroll_position: usize,
    /// Maximum scroll position
    max_scroll: usize,
}

impl HelpScreen {
    /// Creates a new help screen with default shortcuts
    pub fn new() -> Self {
        let sections = vec![
            HelpSection {
                title: "Global".to_string(),
                shortcuts: vec![
                    Shortcut {
                        keys: "Ctrl+Q / Esc".to_string(),
                        action: "Quit / Exit".to_string(),
                    },
                    Shortcut {
                        keys: "? / F1".to_string(),
                        action: "Show this help".to_string(),
                    },
                    Shortcut {
                        keys: ":".to_string(),
                        action: "Enter command mode".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Navigation".to_string(),
                shortcuts: vec![
                    Shortcut {
                        keys: "↑ / k".to_string(),
                        action: "Move up".to_string(),
                    },
                    Shortcut {
                        keys: "↓ / j".to_string(),
                        action: "Move down".to_string(),
                    },
                    Shortcut {
                        keys: "Page Up / Ctrl+B".to_string(),
                        action: "Page up".to_string(),
                    },
                    Shortcut {
                        keys: "Page Down / Ctrl+F".to_string(),
                        action: "Page down".to_string(),
                    },
                    Shortcut {
                        keys: "Home / g".to_string(),
                        action: "Go to top".to_string(),
                    },
                    Shortcut {
                        keys: "End / G".to_string(),
                        action: "Go to bottom".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Operations".to_string(),
                shortcuts: vec![
                    Shortcut {
                        keys: "Enter".to_string(),
                        action: "Confirm / Open".to_string(),
                    },
                    Shortcut {
                        keys: "n / N".to_string(),
                        action: "New password".to_string(),
                    },
                    Shortcut {
                        keys: "/".to_string(),
                        action: "Search".to_string(),
                    },
                    Shortcut {
                        keys: "s / S".to_string(),
                        action: "Sync".to_string(),
                    },
                    Shortcut {
                        keys: "d / D".to_string(),
                        action: "Delete".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Sync".to_string(),
                shortcuts: vec![
                    Shortcut {
                        keys: "Ctrl+S".to_string(),
                        action: "Quick sync".to_string(),
                    },
                    Shortcut {
                        keys: "Ctrl+P".to_string(),
                        action: "Configure provider".to_string(),
                    },
                    Shortcut {
                        keys: "Ctrl+D".to_string(),
                        action: "Manage devices".to_string(),
                    },
                ],
            },
            HelpSection {
                title: "Password Management".to_string(),
                shortcuts: vec![
                    Shortcut {
                        keys: "c / C".to_string(),
                        action: "Copy password".to_string(),
                    },
                    Shortcut {
                        keys: "e / E".to_string(),
                        action: "Edit password".to_string(),
                    },
                    Shortcut {
                        keys: "g / G".to_string(),
                        action: "Generate password".to_string(),
                    },
                    Shortcut {
                        keys: "Ctrl+H".to_string(),
                        action: "Password health".to_string(),
                    },
                ],
            },
        ];

        // Calculate total line count for scroll limits
        let total_lines = Self::calculate_total_lines(&sections);
        let max_scroll = total_lines.saturating_sub(20); // Assume 20 visible lines

        Self {
            sections,
            scroll_position: 0,
            max_scroll,
        }
    }

    /// Returns all help sections
    pub fn get_sections(&self) -> Vec<HelpSection> {
        self.sections.clone()
    }

    /// Returns the current scroll position
    pub fn get_scroll_position(&self) -> usize {
        self.scroll_position
    }

    /// Returns the maximum scroll position
    pub fn get_max_scroll_position(&self) -> usize {
        self.max_scroll
    }

    /// Handles scroll down
    pub fn handle_scroll_down(&mut self) {
        if self.scroll_position < self.max_scroll {
            self.scroll_position += 1;
        }
    }

    /// Handles scroll up
    pub fn handle_scroll_up(&mut self) {
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
        }
    }

    /// Calculates the total number of lines in all sections
    fn calculate_total_lines(sections: &[HelpSection]) -> usize {
        let mut count = 0;

        for section in sections {
            // Section title line
            count += 1;
            // Empty line after title
            count += 1;
            // Shortcut lines
            count += section.shortcuts.len();
            // Empty line after section
            count += 1;
        }

        count
    }

    /// Renders the help screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Title
        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Use ↑↓ or Page Up/Down to scroll, Esc to return",
                Style::default().fg(Color::Gray),
            )),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(4), // Title
                    Constraint::Min(0),    // Help content
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Build help content
        let mut help_lines = vec![];

        for section in &self.sections {
            // Section header
            help_lines.push(Line::from(vec![Span::styled(
                format!("{}:", section.title),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            help_lines.push(Line::from(""));

            // Shortcuts
            for shortcut in &section.shortcuts {
                help_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:20}", shortcut.keys),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}", shortcut.action),
                        Style::default().fg(Color::White),
                    ),
                ]));
            }

            // Empty line between sections
            help_lines.push(Line::from(""));
        }

        let help = Paragraph::new(Text::from(help_lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Shortcuts"),
            )
            .scroll((self.scroll_position as u16, 0));

        frame.render_widget(help, chunks[1]);
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_new() {
        let screen = HelpScreen::new();
        assert_eq!(screen.get_sections().len(), 5);
    }

    #[test]
    fn test_help_default() {
        let screen = HelpScreen::default();
        assert_eq!(screen.get_sections().len(), 5);
    }
}
