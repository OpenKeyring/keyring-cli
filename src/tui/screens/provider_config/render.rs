//! Render implementation for ProviderConfigScreen
//!
//! Contains rendering logic for the provider configuration form.

use super::ProviderConfigScreen;
use crate::cloud::CloudProvider;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

impl ProviderConfigScreen {
    /// Renders the screen to the given frame
    pub fn render_frame(&self, frame: &mut Frame, area: Rect) {
        // Title block
        let block = Block::default()
            .title(format!("  {} Configuration  ", self.provider_name()))
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Layout
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints([
                ratatui::layout::Constraint::Length(4), // Title
                ratatui::layout::Constraint::Min(0),    // Form fields
                ratatui::layout::Constraint::Length(3), // Footer
            ])
            .split(inner);

        // Title
        let title = Paragraph::new(Text::from(vec![Line::from(vec![Span::styled(
            format!("Configure {} Sync", self.provider_name()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )])]))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        // Form fields
        self.render_form_fields(frame, chunks[1]);

        // Footer
        let footer = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::from("Enter: Test Connection  "),
            Span::from("Ctrl+S: Save  "),
            Span::from("Tab: Switch Fields  "),
            Span::from("Esc: Back"),
        ])]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }

    /// Renders the form fields
    fn render_form_fields(&self, frame: &mut Frame, area: Rect) {
        let mut form_lines = vec![];

        for field in &self.fields {
            let display_value = if field.is_password && !field.value.is_empty() {
                "•".repeat(field.value.len())
            } else {
                field.value.clone()
            };

            let is_focused = field.is_focused;

            let line = if is_focused {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", field.label),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!(
                            "[{}]",
                            if display_value.is_empty() {
                                " "
                            } else {
                                &display_value
                            }
                        ),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", field.label),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!(
                            "[{}]",
                            if display_value.is_empty() {
                                " "
                            } else {
                                &display_value
                            }
                        ),
                        Style::default().fg(Color::White),
                    ),
                ])
            };

            form_lines.push(line);
            form_lines.push(Line::from("")); // Empty line between fields
        }

        let form = Paragraph::new(Text::from(form_lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Configuration "),
        );

        frame.render_widget(form, area);
    }

    /// Get provider display name
    fn provider_name(&self) -> &'static str {
        match self.provider {
            CloudProvider::ICloud => "iCloud",
            CloudProvider::Dropbox => "Dropbox",
            CloudProvider::GDrive => "Google Drive",
            CloudProvider::OneDrive => "OneDrive",
            CloudProvider::WebDAV => "WebDAV",
            CloudProvider::SFTP => "SFTP",
            CloudProvider::AliyunDrive => "Aliyun Drive",
            CloudProvider::AliyunOSS => "Aliyun OSS",
            CloudProvider::TencentCOS => "Tencent COS",
            CloudProvider::HuaweiOBS => "Huawei OBS",
            CloudProvider::UpYun => "UpYun",
        }
    }
}
