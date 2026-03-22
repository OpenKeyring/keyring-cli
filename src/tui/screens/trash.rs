//! Trash Screen
//!
//! Displays deleted passwords with options to restore or permanently delete.

use crate::tui::components::ConfirmAction;
use crate::tui::models::password::PasswordRecord;
use crate::tui::state::AppState;
use crate::tui::traits::{
    Action, Component, ComponentId, HandleResult, Interactive, Render, ScreenType,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Trash screen for managing deleted passwords
pub struct TrashScreen {
    id: ComponentId,
    /// Deleted passwords list
    deleted_passwords: Vec<PasswordRecord>,
    /// Currently highlighted index
    selected_index: usize,
    /// Warning threshold (days)
    warning_days: u32,
}

impl TrashScreen {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(0),
            deleted_passwords: Vec::new(),
            selected_index: 0,
            warning_days: 25,
        }
    }

    /// Load deleted passwords from state
    pub fn load_from_state(&mut self, state: &AppState) {
        self.deleted_passwords = state
            .all_passwords()
            .iter()
            .filter(|p| p.is_deleted)
            .cloned()
            .collect();

        // Clamp selection
        if self.selected_index >= self.deleted_passwords.len() && !self.deleted_passwords.is_empty()
        {
            self.selected_index = self.deleted_passwords.len() - 1;
        }
    }

    /// Get currently selected password
    pub fn selected(&self) -> Option<&PasswordRecord> {
        self.deleted_passwords.get(self.selected_index)
    }

    /// Handle key events with state
    pub fn handle_key_with_state(&mut self, key: KeyEvent, state: &mut AppState) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.deleted_passwords.is_empty()
                    && self.selected_index < self.deleted_passwords.len() - 1
                {
                    self.selected_index += 1;
                }
                HandleResult::Consumed
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                HandleResult::Consumed
            }
            KeyCode::Char('r') => {
                // Restore selected
                if let Some(password) = self.selected().cloned() {
                    self.restore_password(&password, state);
                    self.load_from_state(state);
                    state.add_notification(
                        &format!("Restored: {}", password.name),
                        crate::tui::traits::NotificationLevel::Success,
                    );
                }
                HandleResult::Consumed
            }
            KeyCode::Char('D') => {
                // Permanent delete - needs confirmation
                if let Some(password) = self.selected().cloned() {
                    return HandleResult::Action(Action::OpenScreen(ScreenType::ConfirmDialog(
                        ConfirmAction::PermanentDelete(password.id.clone()),
                    )));
                }
                HandleResult::Ignored
            }
            KeyCode::Char('a') => {
                // Empty all - needs confirmation
                if !self.deleted_passwords.is_empty() {
                    return HandleResult::Action(Action::OpenScreen(ScreenType::ConfirmDialog(
                        ConfirmAction::EmptyTrash,
                    )));
                }
                HandleResult::Ignored
            }
            KeyCode::Esc => HandleResult::Action(Action::CloseScreen),
            _ => HandleResult::Ignored,
        }
    }

    fn restore_password(&self, password: &PasswordRecord, state: &mut AppState) {
        let mut updated = password.clone();
        updated.is_deleted = false;
        updated.deleted_at = None;
        state.update_password_in_cache(updated);
    }

    /// Render the trash screen
    pub fn render_frame(&mut self, frame: &mut Frame, area: Rect, state: &AppState) {
        // Reload data
        self.load_from_state(state);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(2), // Info
                Constraint::Min(0),    // List
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Line::from(Span::styled(
            "🗑️ Trash",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
        frame.render_widget(title, chunks[0]);

        // Info
        let info = Paragraph::new(Line::from(Span::styled(
            "Deleted passwords are retained for 30 days before permanent deletion.",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(info, chunks[1]);

        // List
        self.render_list(frame, chunks[2]);

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                "[j/k]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Restore  "),
            Span::styled(
                "[D]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Delete  "),
            Span::styled(
                "[a]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Empty all  "),
            Span::styled(
                "[Esc]",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Back"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[3]);
    }

    fn render_list(&self, frame: &mut Frame, area: Rect) {
        if self.deleted_passwords.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "Trash is empty",
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let mut lines = Vec::new();

        // Header
        lines.push(Line::from(vec![
            Span::styled(format!("{:30}", "Name"), Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:25}", "Username"),
                Style::default().fg(Color::Gray),
            ),
            Span::styled("Deleted ago", Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from("─".repeat(area.width as usize)));

        for (i, password) in self.deleted_passwords.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "> " } else { "  " };

            // Calculate days since deletion
            let days_ago = password
                .deleted_at
                .map(|d| (chrono::Utc::now() - d).num_days() as u32)
                .unwrap_or(0);

            let warning = if days_ago >= self.warning_days {
                " ⚠️"
            } else {
                ""
            };

            let name = format!("{}{}", prefix, password.name);
            let username = password.username.as_deref().unwrap_or("-");
            let deleted_str = format!("{} days ago{}", days_ago, warning);

            lines.push(Line::from(vec![
                Span::styled(format!("{:30}", name), style),
                Span::styled(format!("{:25}", username), style),
                Span::styled(
                    deleted_str,
                    if days_ago >= self.warning_days {
                        Style::default().fg(Color::Yellow)
                    } else {
                        style
                    },
                ),
            ]));
        }

        let list = Paragraph::new(lines);
        frame.render_widget(list, area);
    }
}

impl Default for TrashScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for TrashScreen {
    fn id(&self) -> ComponentId {
        self.id
    }
    fn can_focus(&self) -> bool {
        true
    }
}

impl Interactive for TrashScreen {
    fn handle_key(&mut self, _key: KeyEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Render for TrashScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title(" Trash ");
        block.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trash_screen_new() {
        let screen = TrashScreen::new();
        assert_eq!(screen.selected_index, 0);
        assert!(screen.deleted_passwords.is_empty());
    }

    #[test]
    fn test_trash_screen_navigation() {
        use crossterm::event::KeyModifiers;

        let mut screen = TrashScreen::new();
        let mut state = AppState::new();

        // Add some deleted passwords
        let mut p1 = PasswordRecord::new("1", "Password 1", "pass1");
        p1.is_deleted = true;
        p1.deleted_at = Some(chrono::Utc::now() - chrono::Duration::days(5));

        let mut p2 = PasswordRecord::new("2", "Password 2", "pass2");
        p2.is_deleted = true;
        p2.deleted_at = Some(chrono::Utc::now() - chrono::Duration::days(10));

        state.refresh_password_cache(vec![p1, p2]);
        screen.load_from_state(&state);

        assert_eq!(screen.deleted_passwords.len(), 2);
        assert_eq!(screen.selected_index, 0);

        // Press 'j' to move down
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(screen.selected_index, 1);

        // Press 'k' to move up
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty()),
            &mut state,
        );
        assert_eq!(screen.selected_index, 0);
    }

    #[test]
    fn test_trash_screen_restore() {
        use crossterm::event::KeyModifiers;

        let mut screen = TrashScreen::new();
        let mut state = AppState::new();

        let mut p1 = PasswordRecord::new("1", "Password 1", "pass1");
        p1.is_deleted = true;
        p1.deleted_at = Some(chrono::Utc::now());

        state.refresh_password_cache(vec![p1]);
        screen.load_from_state(&state);

        assert!(screen.selected().unwrap().is_deleted);

        // Press 'r' to restore
        screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty()),
            &mut state,
        );

        // Check password is restored
        let restored = state.get_password_by_str("1").unwrap();
        assert!(!restored.is_deleted);
        assert!(restored.deleted_at.is_none());
    }

    #[test]
    fn test_trash_screen_esc_closes() {
        use crossterm::event::KeyModifiers;

        let mut screen = TrashScreen::new();
        let mut state = AppState::new();

        let result = screen.handle_key_with_state(
            KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
            &mut state,
        );

        assert!(matches!(result, HandleResult::Action(Action::CloseScreen)));
    }
}
