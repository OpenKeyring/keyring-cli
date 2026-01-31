//! TUI Tag Configuration Widget
//!
//! Interactive widget for selecting credential tags in the terminal UI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::tags::config::{EnvTag, RiskTag, TagConfig};

/// Focus area for the tag configuration widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFocus {
    /// Focus on environment tag selection
    Env,
    /// Focus on risk tag selection
    Risk,
    /// Focus on advanced options (custom tags)
    Advanced,
    /// Focus on buttons
    Buttons,
}

/// Tag configuration widget for TUI
pub struct TagConfigWidget {
    /// Credential name being configured
    pub credential_name: String,
    /// Tag configuration state
    config: TagConfig,
    /// Selected environment tag index (0=dev, 1=test, 2=staging, 3=prod)
    pub selected_env: Option<usize>,
    /// Selected risk tag index (0=low, 1=medium, 2=high)
    pub selected_risk: Option<usize>,
    /// Whether to show advanced options
    pub show_advanced: bool,
    /// Current focus area
    focus: TagFocus,
    /// Selected custom tag index (for advanced section)
    pub selected_custom: Option<usize>,
}

impl TagConfigWidget {
    /// Create a new tag configuration widget
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential being configured
    pub fn new(credential_name: String) -> Self {
        Self {
            credential_name,
            config: TagConfig {
                env: None,
                risk: None,
                custom: Vec::new(),
            },
            selected_env: None,
            selected_risk: None,
            show_advanced: false,
            focus: TagFocus::Env,
            selected_custom: None,
        }
    }

    /// Create a new widget with existing tag configuration
    ///
    /// # Arguments
    /// * `credential_name` - Name of the credential being configured
    /// * `config` - Existing tag configuration to load
    pub fn with_config(credential_name: String, config: TagConfig) -> Self {
        let selected_env = config.env.and_then(|env| match env {
            EnvTag::Dev => Some(0),
            EnvTag::Test => Some(1),
            EnvTag::Staging => Some(2),
            EnvTag::Prod => Some(3),
        });

        let selected_risk = config.risk.and_then(|risk| match risk {
            RiskTag::Low => Some(0),
            RiskTag::Medium => Some(1),
            RiskTag::High => Some(2),
        });

        Self {
            credential_name,
            config,
            selected_env,
            selected_risk,
            show_advanced: false,
            focus: TagFocus::Env,
            selected_custom: None,
        }
    }

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
    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                "编辑凭证标签: ",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &self.credential_name,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
    fn draw_env_tags(&self, f: &mut Frame, area: Rect) {
        let env_options = [
            (EnvTag::Dev, "dev (开发环境)"),
            (EnvTag::Test, "test (测试环境)"),
            (EnvTag::Staging, "staging (预发布环境)"),
            (EnvTag::Prod, "prod (生产环境) ⚠️"),
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
                    .title(" 环境标签 (Environment) [单选] ")
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
    fn draw_risk_tags(&self, f: &mut Frame, area: Rect) {
        let risk_options = [
            (RiskTag::Low, "low (低风险)"),
            (RiskTag::Medium, "medium (中风险)"),
            (RiskTag::High, "high (高风险) ⚠️"),
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
                    .title(" 风险标签 (Risk Level) [单选] ")
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
    fn draw_advanced(&self, f: &mut Frame, area: Rect) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    "自定义标签 (Custom Tags)",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("格式: "),
                Span::styled("key:value", Style::default().fg(Color::Yellow)),
                Span::raw(" (例如: "),
                Span::styled("category:database", Style::default().fg(Color::Green)),
                Span::raw(")"),
            ]),
            Line::from(""),
        ];

        if self.config.custom.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("暂无自定义标签", Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "[A] 添加自定义标签",
                    Style::default().fg(Color::Green),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("已添加的标签:", Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(""));

            for (i, tag) in self.config.custom.iter().enumerate() {
                let selected = self.selected_custom == Some(i);
                let focused = self.focus == TagFocus::Advanced;

                let prefix = if selected {
                    "►"
                } else {
                    " "
                };

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
                Span::styled("[A] 添加  ", Style::default().fg(Color::Green)),
                Span::styled("[Enter] 选择", Style::default().fg(Color::Cyan)),
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
                    .title(" 高级选项 (Advanced Options) ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    /// Draw the buttons section
    fn draw_buttons(&self, f: &mut Frame, area: Rect) {
        let focused = self.focus == TagFocus::Buttons;
        let border_style = if focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let text = vec![
            Line::from(vec![
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
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }

    /// Handle key up event
    pub fn on_key_up(&mut self) {
        match self.focus {
            TagFocus::Env => {
                if let Some(ref mut idx) = self.selected_env {
                    *idx = if *idx == 0 { 3 } else { *idx - 1 };
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if let Some(ref mut idx) = self.selected_risk {
                    *idx = if *idx == 0 { 2 } else { *idx - 1 };
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                if !self.config.custom.is_empty() {
                    if let Some(ref mut idx) = self.selected_custom {
                        *idx = if *idx == 0 {
                            self.config.custom.len() - 1
                        } else {
                            *idx - 1
                        };
                    } else {
                        self.selected_custom = Some(0);
                    }
                }
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle key down event
    pub fn on_key_down(&mut self) {
        match self.focus {
            TagFocus::Env => {
                if let Some(ref mut idx) = self.selected_env {
                    *idx = (*idx + 1) % 4;
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if let Some(ref mut idx) = self.selected_risk {
                    *idx = (*idx + 1) % 3;
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                if !self.config.custom.is_empty() {
                    if let Some(ref mut idx) = self.selected_custom {
                        *idx = (*idx + 1) % self.config.custom.len();
                    } else {
                        self.selected_custom = Some(0);
                    }
                }
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle key left event
    pub fn on_key_left(&mut self) {
        match self.focus {
            TagFocus::Risk => {
                self.focus = TagFocus::Env;
            }
            TagFocus::Advanced => {
                self.focus = TagFocus::Risk;
            }
            TagFocus::Buttons => {
                if self.show_advanced {
                    self.focus = TagFocus::Advanced;
                } else {
                    self.focus = TagFocus::Risk;
                }
            }
            TagFocus::Env => {}
        }
    }

    /// Handle key right event
    pub fn on_key_right(&mut self) {
        match self.focus {
            TagFocus::Env => {
                self.focus = TagFocus::Risk;
            }
            TagFocus::Risk => {
                if self.show_advanced {
                    self.focus = TagFocus::Advanced;
                } else {
                    self.focus = TagFocus::Buttons;
                }
            }
            TagFocus::Advanced => {
                self.focus = TagFocus::Buttons;
            }
            TagFocus::Buttons => {}
        }
    }

    /// Handle select/toggle event (Enter or Space)
    pub fn on_select(&mut self) {
        match self.focus {
            TagFocus::Env => {
                // Toggle selection
                if self.selected_env.is_some() {
                    // Already selected, could deselect or keep
                    // For now, keep selection
                } else {
                    self.selected_env = Some(0);
                }
                self.update_config();
            }
            TagFocus::Risk => {
                if self.selected_risk.is_some() {
                    // Already selected
                } else {
                    self.selected_risk = Some(0);
                }
                self.update_config();
            }
            TagFocus::Advanced => {
                // Select a custom tag (for deletion)
                if self.selected_custom.is_none() && !self.config.custom.is_empty() {
                    self.selected_custom = Some(0);
                }
            }
            TagFocus::Buttons => {
                // Trigger save action (handled by caller)
            }
        }
    }

    /// Toggle advanced options visibility
    pub fn toggle_advanced(&mut self) {
        self.show_advanced = !self.show_advanced;
        if self.show_advanced {
            self.focus = TagFocus::Advanced;
        } else {
            self.focus = TagFocus::Risk;
        }
    }

    /// Add a custom tag
    pub fn add_custom_tag(&mut self, tag: String) {
        if !tag.is_empty() && !self.config.custom.contains(&tag) {
            self.config.custom.push(tag);
            self.selected_custom = Some(self.config.custom.len() - 1);
        }
    }

    /// Remove the selected custom tag
    pub fn remove_selected_custom_tag(&mut self) {
        if let Some(idx) = self.selected_custom {
            if idx < self.config.custom.len() {
                self.config.custom.remove(idx);
                if self.config.custom.is_empty() {
                    self.selected_custom = None;
                } else if idx >= self.config.custom.len() {
                    self.selected_custom = Some(self.config.custom.len() - 1);
                }
            }
        }
    }

    /// Get the current tag configuration
    pub fn config(&self) -> &TagConfig {
        &self.config
    }

    /// Take the tag configuration (consuming self)
    pub fn into_config(self) -> TagConfig {
        self.config
    }

    /// Get the current focus area
    pub fn focus(&self) -> TagFocus {
        self.focus
    }

    /// Set the focus area
    pub fn set_focus(&mut self, focus: TagFocus) {
        self.focus = focus;
    }

    /// Check if configuration is ready to save
    pub fn can_save(&self) -> bool {
        // Require at least env tag to be set
        self.config.env.is_some()
    }

    /// Update the internal config from selections
    fn update_config(&mut self) {
        self.config.env = self.selected_env.and_then(|idx| match idx {
            0 => Some(EnvTag::Dev),
            1 => Some(EnvTag::Test),
            2 => Some(EnvTag::Staging),
            3 => Some(EnvTag::Prod),
            _ => None,
        });

        self.config.risk = self.selected_risk.and_then(|idx| match idx {
            0 => Some(RiskTag::Low),
            1 => Some(RiskTag::Medium),
            2 => Some(RiskTag::High),
            _ => None,
        });
    }
}

impl Default for TagConfigWidget {
    fn default() -> Self {
        Self::new("Unnamed Credential".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_default() {
        let widget = TagConfigWidget::default();
        assert_eq!(widget.credential_name, "Unnamed Credential");
        assert!(widget.config.env.is_none());
        assert!(widget.config.risk.is_none());
        assert!(widget.config.custom.is_empty());
    }

    #[test]
    fn test_widget_new() {
        let widget = TagConfigWidget::new("test-credential".to_string());
        assert_eq!(widget.credential_name, "test-credential");
        assert_eq!(widget.focus, TagFocus::Env);
        assert!(!widget.show_advanced);
    }

    #[test]
    fn test_widget_with_config() {
        let config = TagConfig {
            env: Some(EnvTag::Test),
            risk: Some(RiskTag::Medium),
            custom: vec!["custom:tag".to_string()],
        };

        let widget = TagConfigWidget::with_config("test".to_string(), config);
        assert_eq!(widget.selected_env, Some(1));
        assert_eq!(widget.selected_risk, Some(1));
        assert_eq!(widget.config.custom.len(), 1);
    }

    #[test]
    fn test_on_key_down_env() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.selected_env = Some(0);

        widget.on_key_down();
        assert_eq!(widget.selected_env, Some(1));

        widget.on_key_down();
        assert_eq!(widget.selected_env, Some(2));

        widget.on_key_down();
        assert_eq!(widget.selected_env, Some(3));

        widget.on_key_down();
        assert_eq!(widget.selected_env, Some(0)); // Wrap around
    }

    #[test]
    fn test_on_key_up_env() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.selected_env = Some(3);

        widget.on_key_up();
        assert_eq!(widget.selected_env, Some(2));

        widget.on_key_up();
        assert_eq!(widget.selected_env, Some(1));

        widget.on_key_up();
        assert_eq!(widget.selected_env, Some(0));

        widget.on_key_up();
        assert_eq!(widget.selected_env, Some(3)); // Wrap around
    }

    #[test]
    fn test_on_key_down_risk() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.focus = TagFocus::Risk;
        widget.selected_risk = Some(0);

        widget.on_key_down();
        assert_eq!(widget.selected_risk, Some(1));

        widget.on_key_down();
        assert_eq!(widget.selected_risk, Some(2));

        widget.on_key_down();
        assert_eq!(widget.selected_risk, Some(0)); // Wrap around
    }

    #[test]
    fn test_toggle_advanced() {
        let mut widget = TagConfigWidget::new("test".to_string());
        assert!(!widget.show_advanced);

        widget.toggle_advanced();
        assert!(widget.show_advanced);
        assert_eq!(widget.focus, TagFocus::Advanced);

        widget.toggle_advanced();
        assert!(!widget.show_advanced);
        assert_eq!(widget.focus, TagFocus::Risk);
    }

    #[test]
    fn test_add_custom_tag() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.show_advanced = true;

        widget.add_custom_tag("category:database".to_string());
        assert_eq!(widget.config.custom.len(), 1);
        assert_eq!(widget.selected_custom, Some(0));

        // Try adding duplicate
        widget.add_custom_tag("category:database".to_string());
        assert_eq!(widget.config.custom.len(), 1);

        // Add another
        widget.add_custom_tag("owner:team-a".to_string());
        assert_eq!(widget.config.custom.len(), 2);
    }

    #[test]
    fn test_remove_custom_tag() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.show_advanced = true;
        widget.config.custom = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        widget.selected_custom = Some(1);

        widget.remove_selected_custom_tag();
        assert_eq!(widget.config.custom.len(), 2);
        assert_eq!(widget.config.custom, vec!["tag1".to_string(), "tag3".to_string()]);
        assert_eq!(widget.selected_custom, Some(1)); // Still at index 1

        widget.remove_selected_custom_tag();
        assert_eq!(widget.config.custom.len(), 1);
        assert_eq!(widget.selected_custom, Some(0));
    }

    #[test]
    fn test_on_key_left_right() {
        let mut widget = TagConfigWidget::new("test".to_string());
        assert_eq!(widget.focus, TagFocus::Env);

        widget.on_key_right();
        assert_eq!(widget.focus, TagFocus::Risk);

        widget.on_key_right();
        assert_eq!(widget.focus, TagFocus::Buttons);

        widget.on_key_left();
        assert_eq!(widget.focus, TagFocus::Risk);

        widget.on_key_left();
        assert_eq!(widget.focus, TagFocus::Env);
    }

    #[test]
    fn test_update_config() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.selected_env = Some(2);
        widget.selected_risk = Some(1);
        widget.update_config();

        assert_eq!(widget.config.env, Some(EnvTag::Staging));
        assert_eq!(widget.config.risk, Some(RiskTag::Medium));
    }

    #[test]
    fn test_can_save() {
        let mut widget = TagConfigWidget::new("test".to_string());
        assert!(!widget.can_save());

        widget.selected_env = Some(0);
        widget.update_config();
        assert!(widget.can_save());
    }

    #[test]
    fn test_into_config() {
        let mut widget = TagConfigWidget::new("test".to_string());
        widget.selected_env = Some(1);
        widget.selected_risk = Some(2);
        widget.add_custom_tag("custom:tag".to_string());
        widget.update_config();

        let config = widget.into_config();
        assert_eq!(config.env, Some(EnvTag::Test));
        assert_eq!(config.risk, Some(RiskTag::High));
        assert_eq!(config.custom.len(), 1);
    }
}
