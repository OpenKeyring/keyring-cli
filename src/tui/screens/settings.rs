//! Settings Screen
//!
//! TUI screen for viewing and modifying application settings.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Action that can be triggered from the settings screen
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    /// Change master password
    ChangePassword,
    /// Configure biometric unlock
    BiometricUnlock,
    /// View sync status
    SyncStatus,
    /// Configure sync provider
    ConfigureProvider,
    /// Manage devices
    ManageDevices,
    /// Toggle auto-sync
    ToggleAutoSync,
    /// Toggle file monitoring
    ToggleFileMonitoring,
    /// Adjust debounce time
    AdjustDebounce,
}

/// A single settings item
#[derive(Debug, Clone)]
pub struct SettingsItem {
    /// Display label
    pub label: String,
    /// Current value (e.g., "On", "Off", "5s")
    pub value: String,
    /// Whether this item can be toggled
    pub toggleable: bool,
}

/// A settings section containing multiple items
#[derive(Debug, Clone)]
pub struct SettingsSection {
    /// Section title
    pub title: String,
    /// Items in this section
    pub items: Vec<SettingsItem>,
}

/// Settings screen
#[derive(Debug, Clone)]
pub struct SettingsScreen {
    /// Settings sections
    sections: Vec<SettingsSection>,
    /// Currently selected section index
    selected_section: usize,
    /// Currently selected item index within the section
    selected_item: usize,
    /// Actual device count (for sync section)
    device_count: usize,
    /// Actual sync status
    sync_status: String,
    /// Actual provider name
    provider_name: String,
}

impl SettingsScreen {
    /// Creates a new settings screen with default settings
    pub fn new() -> Self {
        let sections = vec![
            SettingsSection {
                title: "Security".to_string(),
                items: vec![
                    SettingsItem {
                        label: "Change Password".to_string(),
                        value: String::new(),
                        toggleable: false,
                    },
                    SettingsItem {
                        label: "Biometric Unlock".to_string(),
                        value: "Off".to_string(),
                        toggleable: true,
                    },
                ],
            },
            SettingsSection {
                title: "Sync".to_string(),
                items: vec![
                    SettingsItem {
                        label: "Status".to_string(),
                        value: "Unsynced".to_string(),
                        toggleable: false,
                    },
                    SettingsItem {
                        label: "Provider".to_string(),
                        value: "None".to_string(),
                        toggleable: false,
                    },
                    SettingsItem {
                        label: "Devices".to_string(),
                        value: "1 device".to_string(),
                        toggleable: false,
                    },
                    SettingsItem {
                        label: "Configure".to_string(),
                        value: String::new(),
                        toggleable: false,
                    },
                ],
            },
            SettingsSection {
                title: "Sync Options".to_string(),
                items: vec![
                    SettingsItem {
                        label: "Auto-sync".to_string(),
                        value: "Off".to_string(),
                        toggleable: true,
                    },
                    SettingsItem {
                        label: "File Monitoring".to_string(),
                        value: "Off".to_string(),
                        toggleable: true,
                    },
                    SettingsItem {
                        label: "Debounce".to_string(),
                        value: "5s".to_string(),
                        toggleable: false,
                    },
                ],
            },
        ];

        Self {
            sections,
            selected_section: 0,
            selected_item: 0,
            device_count: 1,
            sync_status: "Unsynced".to_string(),
            provider_name: "None".to_string(),
        }
    }

    /// Creates a new settings screen with actual data
    pub fn with_data(
        device_count: usize,
        sync_status: &str,
        provider_name: &str,
    ) -> Self {
        let mut screen = Self::new();
        screen.device_count = device_count;
        screen.sync_status = sync_status.to_string();
        screen.provider_name = provider_name.to_string();
        screen.update_sync_section();
        screen
    }

    /// Update the sync section with actual data
    fn update_sync_section(&mut self) {
        // Find and update the Sync section
        for section in &mut self.sections {
            if section.title == "Sync" {
                for item in &mut section.items {
                    if item.label == "Devices" {
                        item.value = format!("{} device{}", self.device_count,
                            if self.device_count == 1 { "" } else { "s" });
                    } else if item.label == "Status" {
                        item.value = self.sync_status.clone();
                    } else if item.label == "Provider" {
                        item.value = self.provider_name.clone();
                    }
                }
                break;
            }
        }
    }

    /// Returns all settings sections
    pub fn get_sections(&self) -> Vec<SettingsSection> {
        self.sections.clone()
    }

    /// Returns the currently selected section index
    pub fn get_selected_section_index(&self) -> usize {
        self.selected_section
    }

    /// Returns the currently selected item index
    pub fn get_selected_item_index(&self) -> usize {
        self.selected_item
    }

    /// Returns the total number of items across all sections
    pub fn get_total_item_count(&self) -> usize {
        self.sections.iter().map(|s| s.items.len()).sum()
    }

    /// Returns the currently selected item, if any
    pub fn get_selected_item(&self) -> Option<SettingsItem> {
        self.sections
            .get(self.selected_section)
            .and_then(|section| section.items.get(self.selected_item))
            .cloned()
    }

    /// Handles down arrow navigation
    pub fn handle_down(&mut self) {
        let current_section = &self.sections[self.selected_section];

        // Move to next item in current section
        if self.selected_item < current_section.items.len() - 1 {
            self.selected_item += 1;
        } else if self.selected_section < self.sections.len() - 1 {
            // Move to first item of next section
            self.selected_section += 1;
            self.selected_item = 0;
        } else {
            // Wrap to beginning
            self.selected_section = 0;
            self.selected_item = 0;
        }
    }

    /// Handles up arrow navigation
    pub fn handle_up(&mut self) {
        // Move to previous item in current section
        if self.selected_item > 0 {
            self.selected_item -= 1;
        } else if self.selected_section > 0 {
            // Move to last item of previous section
            self.selected_section -= 1;
            self.selected_item = self.sections[self.selected_section].items.len() - 1;
        } else {
            // Wrap to end
            self.selected_section = self.sections.len() - 1;
            self.selected_item = self.sections[self.selected_section].items.len() - 1;
        }
    }

    /// Handles Enter key - returns the appropriate action
    pub fn handle_enter(&mut self) -> Option<SettingsAction> {
        let section = &self.sections[self.selected_section];
        let item = &section.items[self.selected_item];

        match (section.title.as_str(), item.label.as_str()) {
            ("Security", "Change Password") => Some(SettingsAction::ChangePassword),
            ("Security", "Biometric Unlock") => Some(SettingsAction::BiometricUnlock),
            ("Sync", "Status") => Some(SettingsAction::SyncStatus),
            ("Sync", "Provider") => Some(SettingsAction::ConfigureProvider),
            ("Sync", "Devices") => Some(SettingsAction::ManageDevices),
            ("Sync", "Configure") => Some(SettingsAction::ConfigureProvider),
            ("Sync Options", "Auto-sync") => Some(SettingsAction::ToggleAutoSync),
            ("Sync Options", "File Monitoring") => Some(SettingsAction::ToggleFileMonitoring),
            ("Sync Options", "Debounce") => Some(SettingsAction::AdjustDebounce),
            _ => None,
        }
    }

    /// Handles toggling a boolean option
    pub fn handle_toggle(&mut self) -> Option<bool> {
        let section = &mut self.sections[self.selected_section];
        let item = &mut section.items[self.selected_item];

        if item.toggleable {
            if item.value == "On" {
                item.value = "Off".to_string();
                Some(false)
            } else {
                item.value = "On".to_string();
                Some(true)
            }
        } else {
            None
        }
    }

    /// Renders the settings screen
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Title
        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "设置 / Settings",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "使用 ↑↓ 导航，Enter 确认，Esc 返回",
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
                    Constraint::Min(0),    // Settings content
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(title, chunks[0]);

        // Settings sections
        let mut settings_lines = vec![];

        for (section_idx, section) in self.sections.iter().enumerate() {
            // Section header
            settings_lines.push(Line::from(vec![
                Span::styled(
                    format!("{}:", section.title),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            settings_lines.push(Line::from(""));

            // Section items
            for (item_idx, item) in section.items.iter().enumerate() {
                let is_selected = section_idx == self.selected_section && item_idx == self.selected_item;

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let value_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let line = if item.value.is_empty() {
                    Line::from(vec![Span::styled(
                        format!("  {}", item.label),
                        style,
                    )])
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!("  {}: ", item.label),
                            style,
                        ),
                        Span::styled(item.value.clone(), value_style),
                    ])
                };

                settings_lines.push(line);
            }

            // Empty line between sections
            settings_lines.push(Line::from(""));
        }

        let settings = Paragraph::new(Text::from(settings_lines))
            .block(Block::default().borders(Borders::ALL).title("设置项 / Settings"));

        frame.render_widget(settings, chunks[1]);

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("Enter: 打开  "),
            Span::from("↑↓: 导航  "),
            Span::from("Esc: 返回"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, chunks[2]);
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_new() {
        let screen = SettingsScreen::new();
        assert_eq!(screen.get_sections().len(), 3);
    }

    #[test]
    fn test_settings_default() {
        let screen = SettingsScreen::default();
        assert_eq!(screen.get_sections().len(), 3);
    }

    #[test]
    fn test_settings_with_data() {
        let screen = SettingsScreen::with_data(3, "Synced", "WebDAV");
        let sections = screen.get_sections();

        let sync_section = &sections[1]; // Sync is section 1
        assert_eq!(sync_section.title, "Sync");

        let devices_item = &sync_section.items[2];
        assert_eq!(devices_item.label, "Devices");
        assert_eq!(devices_item.value, "3 devices");
    }
}
