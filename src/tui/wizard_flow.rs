//! Complete Wizard Flow Implementation
//!
//! Unified wizard flow implementation that coordinates between different screens
//! to guide users through onboarding process.

use crate::tui::error::{TuiError, TuiResult};
use crate::tui::screens::wizard::{WizardState, WizardStep};
use crate::tui::screens::{WelcomeScreen, PasskeyGenerateScreen, PasskeyImportScreen, PasskeyConfirmScreen, MasterPasswordScreen};
use crate::tui::traits::{Component, Render, Interactive, HandleResult, Screen, ScreenType, ComponentId, AppEvent, ScreenResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::buffer::Buffer;
use std::sync::{Arc, Mutex};

/// Shared context for wizard state management
#[derive(Debug, Clone)]
pub struct WizardContext {
    /// Shared wizard state
    pub state: Arc<Mutex<WizardState>>,
}

impl WizardContext {
    /// Create new wizard context
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(WizardState::new())),
        }
    }

    /// Get current state
    pub fn state(&self) -> std::sync::MutexGuard<WizardState> {
        self.state.lock().unwrap()
    }
}

/// Combined trait for screen components that can be rendered and handled interactively
pub trait ScreenComponent: Render + Interactive + Send {}

// Blanket implementation for any type that implements Render + Interactive + Send
impl<T: Render + Interactive + Send> ScreenComponent for T {}

/// Wizard flow screen that manages the entire onboarding process
pub struct WizardScreen {
    /// Shared context with state
    context: WizardContext,
    /// Current step screen
    current_screen: Box<dyn ScreenComponent>,
    /// Current step display name
    current_step_name: String,
}

impl WizardScreen {
    /// Create a new wizard screen with initial state
    pub fn new() -> Self {
        let context = WizardContext::new();
        let mut state = context.state();

        // Initialize with welcome screen
        state.step = WizardStep::Welcome;
        let step_name = state.step.name().to_string();

        drop(state); // Release lock

        let current_screen: Box<dyn ScreenComponent> = Box::new(WelcomeScreen::new());

        Self {
            context,
            current_screen,
            current_step_name: step_name,
        }
    }

    /// Switch to the appropriate screen based on current step
    fn update_screen(&mut self) {
        let state = self.context.state();
        let step = state.step;
        self.current_step_name = step.name().to_string();
        drop(state); // Release lock

        self.current_screen = match step {
            WizardStep::Welcome => Box::new(WelcomeScreen::new()),
            WizardStep::PasskeyGenerate => {
                let state = self.context.state();
                let mut screen = PasskeyGenerateScreen::new();

                // If we already have words, populate them
                if let Some(words) = &state.passkey_words {
                    screen.set_words(words.clone());
                }

                Box::new(screen)
            },
            WizardStep::PasskeyImport => {
                let state = self.context.state();
                let screen = PasskeyImportScreen::new();

                // If we already have words, populate them
                if let Some(words) = &state.passkey_words {
                    // Since PasskeyImportScreen doesn't have a set_words method,
                    // we'll just create a new instance with the current state
                }

                Box::new(screen)
            },
            WizardStep::PasskeyConfirm => {
                let state = self.context.state();

                // For PasskeyConfirmScreen, we need to pass the words during construction
                let words = if let Some(words) = &state.passkey_words {
                    words.clone()
                } else {
                    vec!["".to_string(); 24] // default empty words if none exist
                };

                let mut screen = PasskeyConfirmScreen::new(words);

                // Set the confirmed state
                if state.confirmed {
                    screen.set_confirmed(true);
                }

                Box::new(screen)
            },
            WizardStep::MasterPassword => {
                let state = self.context.state();
                let screen = MasterPasswordScreen::new();

                // For master password screen, we'll manage the state externally
                Box::new(screen)
            },
            WizardStep::Complete => Box::new(CompletionScreen::new()),
        };
    }

    /// Handle moving to next step
    fn handle_next(&mut self) -> HandleResult {
        let mut state = self.context.state();

        match state.step {
            WizardStep::Welcome => {
                if state.passkey_choice.is_some() {
                    // Generate new passkey if chosen
                    if let Some(crate::tui::screens::WelcomeChoice::GenerateNew) = state.passkey_choice {
                        // Generate new passkey words
                        use crate::crypto::passkey::Passkey;
                        let passkey_result = Passkey::generate(24);

                        match passkey_result {
                            Ok(passkey_gen) => {
                                // Convert passkey to words
                                let words: Vec<String> = passkey_gen.to_words();
                                state.set_passkey_words(words);
                            },
                            Err(_) => {
                                state.set_error("Failed to generate passkey".to_string());
                                return HandleResult::Ignored;
                            }
                        }
                    }
                    state.next();

                    drop(state); // Release lock
                    self.update_screen();

                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            },
            WizardStep::PasskeyGenerate => {
                if state.passkey_words.is_some() {
                    state.next();

                    drop(state); // Release lock
                    self.update_screen();

                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            },
            WizardStep::PasskeyImport => {
                // For import, we need to check the import screen for validation
                // For now, assume if we have words we can proceed
                if state.passkey_words.is_some() {
                    state.next();

                    drop(state); // Release lock
                    self.update_screen();

                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            },
            WizardStep::PasskeyConfirm => {
                if state.confirmed {
                    state.next();

                    drop(state); // Release lock
                    self.update_screen();

                    HandleResult::Consumed
                } else {
                    // Toggle confirmation
                    state.toggle_confirmed();
                    drop(state); // Release lock
                    self.update_screen();
                    HandleResult::NeedsRender
                }
            },
            WizardStep::MasterPassword => {
                if state.master_password.is_some() &&
                   state.master_password.as_ref().unwrap().len() >= 8 {
                    state.next();

                    drop(state); // Release lock
                    self.update_screen();

                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            },
            WizardStep::Complete => {
                // Wizard is complete
                HandleResult::Action(crate::tui::traits::Action::CloseScreen)
            }
        }
    }

    /// Handle going back to previous step
    fn handle_back(&mut self) -> HandleResult {
        let mut state = self.context.state();

        if state.can_go_back() {
            state.back();

            drop(state); // Release lock
            self.update_screen();

            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }

    /// Handle input for current screen and manage transitions
    fn handle_current_screen_input(&mut self, key: KeyEvent) -> HandleResult {
        self.current_screen.handle_key(key)
    }
}

impl Render for WizardScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Create a bordered area for the wizard
        let block = Block::default()
            .title(format!("OpenKeyring Setup - {}", self.current_step_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Render the current step screen
        self.current_screen.render(inner_area, buf);

        // Show progress indicator
        let progress = match self.context.state().step {
            WizardStep::Welcome => 10,
            WizardStep::PasskeyGenerate | WizardStep::PasskeyImport => 30,
            WizardStep::PasskeyConfirm => 60,
            WizardStep::MasterPassword => 80,
            WizardStep::Complete => 100,
        };

        let progress_text = Paragraph::new(
            Line::from(vec![
                Span::styled("Progress: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}%", progress),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                ),
                Span::raw(" | "),
                Span::styled("ESC: Back", Style::default().fg(Color::Blue)),
                Span::raw(" | "),
                Span::styled("ENTER: Next", Style::default().fg(Color::Blue)),
            ])
        ).alignment(ratatui::layout::Alignment::Center);

        // Split area to make space for progress bar at the bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ].as_ref())
            .split(inner_area);

        progress_text.render(chunks[1], buf);
    }
}

impl Interactive for WizardScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Enter => self.handle_next(),
            KeyCode::Esc => self.handle_back(),
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+Q to quit
                HandleResult::Action(crate::tui::traits::Action::Quit)
            }
            _ => self.handle_current_screen_input(key),
        }
    }
}

impl Component for WizardScreen {
    fn id(&self) -> ComponentId {
        ComponentId::new(2000) // Unique ID for WizardScreen
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, event: &AppEvent) -> HandleResult {
        match event {
            AppEvent::Key(key_event) => self.handle_key(*key_event),
            _ => HandleResult::Ignored,
        }
    }
}

impl Screen for WizardScreen {
    fn screen_type(&self) -> ScreenType {
        ScreenType::Wizard
    }

    fn close(&mut self) -> TuiResult<()> {
        // Reset state when closing
        let mut state = self.context.state();
        *state = WizardState::new();
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

    fn result(&self) -> Option<ScreenResult> {
        let state = self.context.state();
        if state.is_complete() {
            Some(ScreenResult::Confirmed)
        } else {
            None
        }
    }
}

/// Completion screen for the wizard flow
struct CompletionScreen {
    message: String,
}

impl CompletionScreen {
    fn new() -> Self {
        Self {
            message: "Setup complete!\n\nYour OpenKeyring is ready to use.\n\nPress Enter to finish.".to_string(),
        }
    }
}

impl Render for CompletionScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Setup Complete!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));

        let paragraph = Paragraph::new(self.message.clone())
            .block(block)
            .style(Style::default().fg(Color::Green))
            .wrap(ratatui::widgets::Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

impl Interactive for CompletionScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.code == KeyCode::Enter {
            HandleResult::Action(crate::tui::traits::Action::CloseScreen)
        } else {
            HandleResult::Ignored
        }
    }
}

impl Component for CompletionScreen {
    fn id(&self) -> ComponentId {
        ComponentId::new(2001) // Unique ID for CompletionScreen
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, _event: &AppEvent) -> HandleResult {
        HandleResult::Ignored
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
        assert_eq!(wizard.context.state().step, WizardStep::Welcome);
    }

    #[test]
    fn test_wizard_screen_next() {
        let mut wizard = WizardScreen::new();

        // Set choice to generate new
        {
            let mut state = wizard.context.state();
            state.set_passkey_choice(crate::tui::screens::WelcomeChoice::GenerateNew);
        }

        // Simulate next action
        let result = wizard.handle_next();
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(wizard.context.state().step, WizardStep::PasskeyGenerate);
    }

    #[test]
    fn test_wizard_screen_back() {
        let mut wizard = WizardScreen::new();

        // Advance to passkey generate
        {
            let mut state = wizard.context.state();
            state.set_passkey_choice(crate::tui::screens::WelcomeChoice::GenerateNew);
            state.set_passkey_words(vec!["test".to_string(); 24]);
            state.next(); // Should be on PasskeyConfirm
        }

        // Go back
        let result = wizard.handle_back();
        assert!(matches!(result, HandleResult::Consumed));
        assert_eq!(wizard.context.state().step, WizardStep::PasskeyGenerate);
    }
}