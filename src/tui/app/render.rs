//! Rendering for TUI application
//!
//! Contains all rendering methods for the TUI.

use super::TuiApp;
use crate::tui::screens::wizard::WizardStep;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

impl TuiApp {
    /// Render the TUI
    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Render the current screen
        match self.current_screen {
            super::types::Screen::Wizard => {
                if let Some(state) = &self.wizard_state {
                    self.render_wizard(frame, size, state);
                }
            }
            super::types::Screen::Unlock => {
                self.unlock_screen.render(frame, size);
            }
            super::types::Screen::Main => {
                self.main_screen.render_frame(frame, size, &self.app_state);
            }
            super::types::Screen::NewPassword => {
                use crate::tui::traits::Render;
                self.new_password_screen.render(size, frame.buffer_mut());
            }
            super::types::Screen::EditPassword => {
                use crate::tui::traits::Render;
                self.edit_password_screen.render(size, frame.buffer_mut());
            }
            super::types::Screen::Trash => {
                self.trash_screen.render_frame(frame, size, &self.app_state);
            }
            super::types::Screen::Settings => {
                self.settings_screen.render(frame, size);
            }
            super::types::Screen::Help => {
                self.help_screen.render(frame, size);
            }
            super::types::Screen::Sync => {
                if let Some(screen) = &self.sync_screen {
                    screen.render(frame, size);
                }
            }
            _ => {
                // Fallback for unhandled screens
                let msg = Paragraph::new("Screen not implemented")
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(msg, size);
            }
        }

        // Render confirm dialog overlay on top of current screen
        if let Some(dialog) = &self.confirm_dialog {
            use crate::tui::traits::Render;
            dialog.render(size, frame.buffer_mut());
        }
    }

    /// Render the wizard screen
    pub(crate) fn render_wizard(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &crate::tui::screens::wizard::WizardState,
    ) {
        use crate::tui::traits::Render;

        match state.step {
            WizardStep::Welcome => {
                self.welcome_screen.render(frame, area);
            }
            WizardStep::MasterPassword => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::MasterPasswordConfirm => {
                // Use MasterPasswordScreen's confirm mode or separate screen
                self.master_password_screen.render(frame, area);
            }
            WizardStep::SecurityNotice => {
                self.security_notice_screen.render(area, frame.buffer_mut());
            }
            WizardStep::PasskeyGenerate => {
                self.passkey_generate_screen.render(frame, area);
            }
            WizardStep::PasskeyVerify => {
                if let Some(screen) = &self.passkey_verify_screen {
                    screen.render(area, frame.buffer_mut());
                }
            }
            WizardStep::PasskeyImport => {
                self.passkey_import_screen.render(frame, area);
            }
            WizardStep::MasterPasswordImport => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::MasterPasswordImportConfirm => {
                self.master_password_screen.render(frame, area);
            }
            WizardStep::PasswordHint => {
                // Simple hint screen - render as paragraph
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("Password Hint"),
                    Line::from(""),
                    Line::from("Your PassKey has been imported successfully."),
                    Line::from(""),
                    Line::from("Make sure to remember your master password."),
                    Line::from(""),
                    Line::from("Press Enter to continue..."),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, area);
            }
            WizardStep::PasswordPolicy => {
                self.password_policy_screen.render(area, frame.buffer_mut());
            }
            WizardStep::ClipboardTimeout => {
                self.clipboard_timeout_screen
                    .render(area, frame.buffer_mut());
            }
            WizardStep::TrashRetention => {
                self.trash_retention_screen.render(area, frame.buffer_mut());
            }
            WizardStep::ImportPasswords => {
                // Optional import screen - for now just show message
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("Import Existing Passwords"),
                    Line::from(""),
                    Line::from("This step is optional."),
                    Line::from(""),
                    Line::from("Press Enter to skip or provide import file..."),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(paragraph, area);
            }
            WizardStep::Complete => {
                // Show completion message with quick start guide
                let paragraph = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        "Setup Complete!",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Quick Start Guide:",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  [n] Create a new password",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [j/k] Navigate through your passwords",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [Enter] View password details",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [c] Copy username  |  [C] Copy password",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::styled(
                        "  [?] Show help anytime",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Enter] to start using OpenKeyring",
                        Style::default().fg(Color::Gray),
                    )),
                ])
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Welcome to OpenKeyring "),
                );

                frame.render_widget(paragraph, area);
            }
        }
    }

    /// Render the statusline
    pub fn render_statusline(&self, width: u16) -> Vec<Span<'_>> {
        let mut spans = Vec::new();

        // Narrow screen (<60 columns): show only sync status
        if width < 60 {
            spans.push(Span::styled(
                format!(" {}", self.sync_status.display()),
                Style::default().fg(Color::DarkGray),
            ));
            return spans;
        }

        // Full statusline for width >= 60 columns
        let width_usize = width as usize;

        // Left: lock status + record count
        let lock_icon = if self.locked { "locked" } else { "unlocked" };
        let left_part = format!("{} {} rec", lock_icon, self.record_count);
        spans.push(Span::styled(left_part, Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" | "));

        // Center-left: sync status
        spans.push(Span::styled(
            self.sync_status.display(),
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::raw(" | "));

        // Center-right: version
        spans.push(Span::styled(
            format!("v{}", self.version),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::raw(" | "));

        // Right: keyboard hints (most important shortcuts)
        let hints = self.get_keyboard_hints(width_usize);
        spans.push(Span::styled(
            hints,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        spans
    }

    /// Get keyboard hints for the statusline
    fn get_keyboard_hints(&self, width: usize) -> String {
        // For very wide screens, show more hints
        if width >= 100 {
            "Ctrl+N new | Ctrl+L list | Ctrl+Q quit".to_string()
        } else if width >= 80 {
            "Ctrl+N new | Ctrl+Q quit".to_string()
        } else {
            "Ctrl+Q quit".to_string()
        }
    }

    /// Render the statusline widget
    fn render_statusline_widget(&self, frame: &mut Frame, area: Rect) {
        let spans = self.render_statusline(area.width);
        let line = Line::from(spans);

        let paragraph = Paragraph::new(Text::from(line)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(paragraph, area);
    }
}

/// Create a centered rectangle within the given area
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
