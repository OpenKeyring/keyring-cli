//! Render implementation for TagConfigWidget
//!
//! Contains all drawing/rendering methods for the tag configuration widget.

use super::TagConfigWidget;
use super::types::TagFocus;
use crate::tui::tags::config::{EnvTag, RiskTag};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

impl TagConfigWidget {
    /// Draw the widget
    ///
    /// # Arguments
    /// * `f` - Frame to render on
    /// * `area` - Area to render in
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        // Calculate constraints based on whether advanced is shown
        let constraints = if self.show_advanced {
            [
                Constraint::Length(3),  // Header
                Constraint::Length(10), // Env tags
                Constraint::Length(10), // Risk tags
                Constraint::Min(10),    // Advanced (expandable)
                Constraint::Length(3),  // Buttons
            ]
        } else {
            [
                Constraint::Length(3),  // Header
                Constraint::Length(10), // Env tags
                Constraint::Length(10), // Risk tags
                Constraint::Length(0),  // Advanced (hidden)
                Constraint::Length(3),  // Buttons
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.as_ref())
            .split(area);

        self.draw_header(f, chunks[0]);
        self.draw_env_tags(f, chunks[1]);
        self.draw_risk_tags(f, chunks[2]);

        if self.show_advanced {
            self.draw_advanced(f, chunks[3]);
        }

        self.draw_buttons(f, chunks[4]);
    }

    /// Draw the header section
    pub(super) fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                "Edit Credential Tags: ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &self.credential_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let paragraph = Paragraph::new(title)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    /// Draw the environment tag selection section
    pub(super) fn draw_env_tags(&self, f: &mut Frame, area: Rect) {
        let env_options = [
            (EnvTag::Dev, "dev (development)"),
            (EnvTag::Test, "test (testing)"),
            (EnvTag::Staging, "staging (pre-production)"),
            (EnvTag::Prod, "prod (production) ⚠️"),
        ];

        let items: Vec<ListItem> = env_options
            .iter()
            .enumerate()
            .map(|(i, (_env, label))| {
                let selected = self.selected_env == Some(i);
                let focused = self.focus == TagFocus::Env;

                let prefix = if selected { "(x)" } else { "( )" };

                let style = if selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if focused {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(format!("{} {}", prefix, label)).style(style)
            })
            .collect();

        let border_style = if self.focus == TagFocus::Env {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Environment [Single] ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(list, area);
    }

    /// Draw the risk tag selection section
    pub(super) fn draw_risk_tags(&self, f: &mut Frame, area: Rect) {
        let risk_options = [
            (RiskTag::Low, "low (low risk)"),
            (RiskTag::Medium, "medium (medium risk)"),
            (RiskTag::High, "high (high risk) ⚠️"),
        ];

        let items: Vec<ListItem> = risk_options
            .iter()
            .enumerate()
            .map(|(i, (_risk, label))| {
                let selected = self.selected_risk == Some(i);
                let focused = self.focus == TagFocus::Risk;

                let prefix = if selected { "(x)" } else { "( )" };

                let style = if selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if focused {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(format!("{} {}", prefix, label)).style(style)
            })
            .collect();

        let border_style = if self.focus == TagFocus::Risk {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Risk Level [Single] ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(list, area);
    }

    /// Draw the advanced options section
    pub(super) fn draw_advanced(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Custom Tags",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Format: "),
                Span::styled("key:value", Style::default().fg(Color::Yellow)),
                Span::raw(" (e.g.: "),
                Span::styled("category:database", Style::default().fg(Color::Green)),
                Span::raw(")"),
            ]),
            Line::from(""),
        ];

        if self.config.custom.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No custom tags",
                Style::default().fg(Color::DarkGray),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "[A] Add custom tag",
                Style::default().fg(Color::Green),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "Added tags:",
                Style::default().fg(Color::White),
            )]));
            lines.push(Line::from(""));

            for (i, tag) in self.config.custom.iter().enumerate() {
                let selected = self.selected_custom == Some(i);
                let focused = self.focus == TagFocus::Advanced;

                let prefix = if selected { "►" } else { " " };

                let style = if selected && focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(prefix, style),
                    Span::raw(" "),
                    Span::styled(tag, style),
                    Span::raw(" "),
                    Span::styled("[Del]", Style::default().fg(Color::Red)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("[A] Add  ", Style::default().fg(Color::Green)),
                Span::styled("[Enter] Select", Style::default().fg(Color::Cyan)),
            ]));
        }

        let border_style = if self.focus == TagFocus::Advanced {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Advanced Options ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    /// Draw the buttons section
    pub(super) fn draw_buttons(&self, f: &mut Frame, area: Rect) {
        let focused = self.focus == TagFocus::Buttons;
        let border_style = if focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let text = vec![Line::from(vec![
            Span::raw(" ["),
            Span::styled(
                "S",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("]ave & Preview  "),
            Span::raw("["),
            Span::styled(
                "A",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("]dvanced  "),
            Span::raw("["),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("] Cancel"),
        ])];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}
