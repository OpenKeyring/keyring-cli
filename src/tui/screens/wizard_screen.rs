//! Wizard Flow Implementation
//!
//! Complete wizard flow implementation with screens for welcome, setup choice,
//! master password, passkey generation/validation, and completion.

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::screens::wizard::{WizardState, WizardStep};
use crate::tui::screens::{WelcomeScreen, PasskeyGenerateScreen, PasskeyImportScreen, PasskeyConfirmScreen, MasterPasswordScreen};
use crate::tui::traits::{Component, Render, Interactive, HandleResult, Screen, ScreenType, ComponentId, AppEvent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::buffer::Buffer;

/// Wizard flow screen that manages the entire onboarding process
#[derive(Debug)]
pub struct WizardScreen {
    /// Current wizard state
    pub state: WizardState,
}

impl WizardScreen {
    /// Create a new wizard screen
    pub fn new() -> Self {
        let state = WizardState::new();

        Self {
            state,
        }
    }

    /// Navigate to the appropriate screen based on current step
    fn render_current_step(&self, frame: &mut ratatui::Frame, area: Rect) {
        match self.state.step {
            WizardStep::Welcome => {
                let welcome_screen = WelcomeScreen::new();
                welcome_screen.render(frame, area);
            }
            WizardStep::PasskeyGenerate => {
                let mut generate_screen = PasskeyGenerateScreen::new();

                // If we already have words, populate them
                if let Some(words) = &self.state.passkey_words {
                    generate_screen.set_words(words.clone());
                }

                generate_screen.render(frame, area);
            }
            WizardStep::PasskeyImport => {
                let mut import_screen = PasskeyImportScreen::new();

                // If we already have words, populate them
                if let Some(words) = &self.state.passkey_words {
                    import_screen.set_words(words.clone());
                }

                import_screen.render(frame, area);
            }
            WizardStep::PasskeyConfirm => {
                let mut confirm_screen = PasskeyConfirmScreen::new();

                // Set the words to confirm
                if let Some(words) = &self.state.passkey_words {
                    confirm_screen.set_words(words.clone());
                }

                // Set the confirmed state
                confirm_screen.set_confirmed(self.state.confirmed);

                confirm_screen.render(frame, area);
            }
            WizardStep::MasterPassword => {
                let mut password_screen = MasterPasswordScreen::new();

                // Set the password if available
                if let Some(password) = &self.state.master_password {
                    password_screen.set_password(password.clone());
                }

                password_screen.render(frame, area);
            }
            WizardStep::Complete => {
                // For now, we'll show a completion message
                let completion_screen = CompletionScreen::new();
                completion_screen.render(frame, area);
            }
        }
    }

    /// Handle next action
    fn handle_next(&mut self) -> HandleResult {
        match self.state.step {
            WizardStep::Welcome => {
                if self.state.passkey_choice.is_some() {
                    // Generate new passkey if chosen
                    if let Some(crate::tui::screens::WelcomeChoice::GenerateNew) = self.state.passkey_choice {
                        // Generate new passkey words
                        use crate::crypto::passkey::Passkey;
                        let passkey = Passkey::generate(24).expect("Could not generate passkey");
                        let words: Vec<String> = passkey.to_phrase().split_whitespace().map(|s| s.to_string()).collect();

                        self.state.set_passkey_words(words);
                    }
                    self.state.next();
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            WizardStep::PasskeyGenerate => {
                // Validate that we have generated words
                if self.state.passkey_words.is_some() {
                    self.state.next();
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            WizardStep::PasskeyImport => {
                // Here we would validate the imported passkey
                if self.state.passkey_words.is_some() {
                    self.state.next();
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            WizardStep::PasskeyConfirm => {
                if self.state.confirmed {
                    self.state.next();
                    HandleResult::Consumed
                } else {
                    // Toggle confirmation
                    self.state.toggle_confirmed();
                    HandleResult::NeedsRender
                }
            }
            WizardStep::MasterPassword => {
                // For this step, we'll move to complete
                if self.state.master_password.is_some() && self.state.master_password.as_ref().unwrap().len() >= 8 {
                    self.state.next();
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            WizardStep::Complete => {
                // Wizard is complete, might want to save settings here
                HandleResult::Action(crate::tui::traits::Action::CloseScreen)
            }
        }
    }

    /// Handle back action
    fn handle_back(&mut self) -> HandleResult {
        if self.state.can_go_back() {
            self.state.back();
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }
}

impl Render for WizardScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Use a Frame to render the current step screen properly
        use ratatui::Frame;

        // Unfortunately, we can't easily render directly to buf with the individual screens
        // So we'll create a simple render for the wizard progress
        let block = Block::default()
            .title(format!("Setup Wizard - {}", self.state.step.name()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Draw step indicator
        let step_indicator = Paragraph::new(Line::from(vec![
            Span::styled("Current Step: ", Style::default().fg(Color::Yellow)),
            Span::styled(self.state.step.name(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | "),
            Span::styled("Progress: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{}%",
                    match self.state.step {
                        WizardStep::Welcome => 10,
                        WizardStep::PasskeyGenerate | WizardStep::PasskeyImport => 30,
                        WizardStep::PasskeyConfirm => 60,
                        WizardStep::MasterPassword => 80,
                        WizardStep::Complete => 100,
                    }
                ),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            ),
        ]));

        // Split area to reserve space for the progress info at the bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ].as_ref())
            .split(inner_area);

        step_indicator.render(chunks[1], buf);
    }
}

impl Interactive for WizardScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Enter => self.handle_next(),
            KeyCode::Esc => self.handle_back(),
            KeyCode::Char(' ') => {
                // Space bar to confirm on confirmation screen
                if self.state.step == WizardStep::PasskeyConfirm {
                    self.state.toggle_confirmed();
                    HandleResult::NeedsRender
                } else {
                    HandleResult::Ignored
                }
            }
            _ => {
                // Handle input for the current step
                match self.state.step {
                    WizardStep::Welcome => {
                        let mut welcome_screen = WelcomeScreen::new();
                        // If we have a choice, apply it
                        if let Some(choice) = self.state.passkey_choice {
                            welcome_screen.set_choice(choice);
                        }
                        // Process the key for the current screen - we can't directly call
                        // the screen's handle_key since we don't store it as a Screen object
                        // So we'll handle navigation based on the state
                        HandleResult::Ignored
                    }
                    WizardStep::MasterPassword => {
                        // We'll assume password input happens externally and gets set via state
                        HandleResult::Ignored
                    }
                    _ => HandleResult::Ignored
                }
            }
        }
    }

    fn handle_mouse(&mut self, _event: crossterm::event::MouseEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Component for WizardScreen {
    fn id(&self) -> ComponentId {
        ComponentId::new(0)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, _event: &AppEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

impl Screen for WizardScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::Wizard
    }

    fn close(&mut self) -> TuiResult<()> {
        // Clean up wizard state
        self.state = WizardState::new();
        Ok(())
    }

    fn is_modal(&self) -> bool {
        true
    }

    fn show_overlay(&self) -> bool {
        true
    }

    fn size(&self, terminal: Rect) -> Rect {
        let width = (terminal.width as f32 * 0.9) as u16;
        let height = (terminal.height as f32 * 0.9) as u16;
        let x = (terminal.width.saturating_sub(width)) / 2;
        let y = (terminal.height.saturating_sub(height)) / 2;
        Rect::new(x, y, width, height)
    }

    fn result(&self) -> Option<crate::tui::traits::screen::ScreenResult> {
        if self.state.is_complete() {
            Some(crate::tui::traits::screen::ScreenResult::Confirmed)
        } else {
            None
        }
    }
}

/// Completion screen for the wizard flow
#[derive(Debug, Clone)]
pub struct CompletionScreen {
    message: String,
}

impl CompletionScreen {
    pub fn new() -> Self {
        Self {
            message: "Setup complete!\n\nYour OpenKeyring is ready to use.\n\nPress Enter to continue.".to_string(),
        }
    }

    /// Render the completion screen
    pub fn render(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::layout::Alignment;

        let block = Block::default()
            .title("🎉 Setup Complete!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        block.render(area, frame.buffer_mut());

        // Create the quick start guide content
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Your OpenKeyring is ready to use!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Quick Start Guide:",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::raw("  [n] Create a new password")),
            Line::from(Span::raw("  [j/k] Navigate through your passwords")),
            Line::from(Span::raw("  [Enter] View password details")),
            Line::from(Span::raw("  [c] Copy username | [C] Copy password")),
            Line::from(Span::raw("  [Space] Toggle password visibility")),
            Line::from(Span::raw("  [?] Show help anytime")),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to start using OpenKeyring",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, inner);
    }
}

impl Default for WizardScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_screen_new() {
        let wizard = WizardScreen::new();
        assert_eq!(wizard.state.step, WizardStep::Welcome);
    }

    #[test]
    fn test_wizard_screen_next() {
        let mut wizard = WizardScreen::new();

        // Set choice to generate new
        wizard.state.set_passkey_choice(crate::tui::screens::WelcomeChoice::GenerateNew);

        // Simulate next action
        let result = wizard.handle_next();
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(wizard.state.step, WizardStep::PasskeyGenerate);
    }

    #[test]
    fn test_wizard_screen_back() {
        let mut wizard = WizardScreen::new();

        // Advance to passkey generate
        wizard.state.set_passkey_choice(crate::tui::screens::WelcomeChoice::GenerateNew);
        wizard.state.set_passkey_words(vec!["test".to_string(); 24]);
        wizard.state.next(); // Should be on PasskeyConfirm

        // Go back
        let result = wizard.handle_back();
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(wizard.state.step, WizardStep::PasskeyGenerate);
    }
}