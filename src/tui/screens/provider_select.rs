//! Cloud Provider Selection Screen
//!
//! TUI screen for selecting from supported cloud storage providers.

use crate::cloud::CloudProvider;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Display information for a cloud provider
#[derive(Debug, Clone)]
pub struct Provider {
    /// Display name (e.g., "iCloud Drive")
    pub name: &'static str,
    /// Keyboard shortcut (1-8)
    pub shortcut: char,
    /// Underlying cloud provider type
    pub provider: CloudProvider,
}

/// Cloud provider selection screen
#[derive(Debug, Clone)]
pub struct ProviderSelectScreen {
    /// List of all available providers
    providers: Vec<Provider>,
    /// Currently selected provider index
    selected_index: usize,
    /// Whether a provider has been selected
    selected: bool,
}

impl ProviderSelectScreen {
    /// Creates a new provider selection screen with all supported providers
    pub fn new() -> Self {
        let providers = vec![
            Provider {
                name: "iCloud Drive",
                shortcut: '1',
                provider: CloudProvider::ICloud,
            },
            Provider {
                name: "Dropbox",
                shortcut: '2',
                provider: CloudProvider::Dropbox,
            },
            Provider {
                name: "Google Drive",
                shortcut: '3',
                provider: CloudProvider::GDrive,
            },
            Provider {
                name: "OneDrive",
                shortcut: '4',
                provider: CloudProvider::OneDrive,
            },
            Provider {
                name: "WebDAV",
                shortcut: '5',
                provider: CloudProvider::WebDAV,
            },
            Provider {
                name: "SFTP",
                shortcut: '6',
                provider: CloudProvider::SFTP,
            },
            Provider {
                name: "阿里云盘",
                shortcut: '7',
                provider: CloudProvider::AliyunDrive,
            },
            Provider {
                name: "阿里云 OSS",
                shortcut: '8',
                provider: CloudProvider::AliyunOSS,
            },
        ];

        Self {
            providers,
            selected_index: 0,
            selected: false,
        }
    }

    /// Returns the list of all providers
    pub fn get_providers(&self) -> &[Provider] {
        &self.providers
    }

    /// Returns the currently selected provider index
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the selected cloud provider, if any
    pub fn get_selected_provider(&self) -> Option<CloudProvider> {
        if self.selected {
            self.providers.get(self.selected_index).map(|p| p.provider)
        } else {
            None
        }
    }

    /// Handles character input for quick provider selection (1-8)
    pub fn handle_char(&mut self, c: char) {
        if let Some(idx) = c.to_digit(10) {
            let idx = (idx as usize) - 1;
            if idx < self.providers.len() {
                self.selected_index = idx;
                self.selected = true;
            }
        }
    }

    /// Handles down arrow navigation
    pub fn handle_down(&mut self) {
        if self.selected_index < self.providers.len() - 1 {
            self.selected_index += 1;
        }
        self.selected = true;
    }

    /// Handles up arrow navigation
    pub fn handle_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.selected = true;
    }

    /// Renders the provider selection screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Title
        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "选择云存储服务 / Select Cloud Storage",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "按数字键 1-8 快速选择，或使用 ↑↓ 导航",
                Style::default().fg(Color::Gray),
            )),
        ]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Length(4), // Title
                    ratatui::layout::Constraint::Min(0),    // Provider list
                    ratatui::layout::Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Provider list
        let items: Vec<ListItem> = self
            .providers
            .iter()
            .enumerate()
            .map(|(i, provider)| {
                let is_selected = i == self.selected_index;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("({}) ", provider.shortcut),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(provider.name, style),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("可用服务 / Available"));

        frame.render_widget(list, chunks[1]);

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("Enter: 确认  "),
            Span::from("Esc: 取消  "),
            Span::from("↑↓: 导航"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }
}

impl Default for ProviderSelectScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new() {
        let screen = ProviderSelectScreen::new();
        assert_eq!(screen.get_providers().len(), 8);
        assert_eq!(screen.get_selected_index(), 0);
        assert_eq!(screen.get_selected_provider(), None);
    }

    #[test]
    fn test_provider_default() {
        let screen = ProviderSelectScreen::default();
        assert_eq!(screen.get_providers().len(), 8);
    }
}
