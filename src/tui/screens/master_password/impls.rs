//! Trait implementations for MasterPasswordScreen
//!
//! Contains implementations of Interactive, Default, WizardStepValidator, and WizardStepScreen traits.

use super::render::render;
use super::MasterPasswordScreen;
use crate::tui::screens::wizard::WizardState;
use crate::tui::traits::{HandleResult, Interactive, WizardStepScreen, WizardStepValidator};
use crossterm::event::KeyCode;
use ratatui::{layout::Rect, Frame};

impl Interactive for MasterPasswordScreen {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> HandleResult {
        match key.code {
            KeyCode::Tab => {
                if self.is_showing_first() {
                    self.next();
                } else {
                    self.back();
                }
                HandleResult::NeedsRender
            }
            KeyCode::BackTab => {
                if !self.is_showing_first() {
                    self.back();
                }
                HandleResult::NeedsRender
            }
            KeyCode::Char(c) => {
                self.handle_char(c);
                HandleResult::NeedsRender
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                HandleResult::NeedsRender
            }
            KeyCode::Enter => {
                if self.can_complete() {
                    HandleResult::Consumed
                } else if self.is_showing_first() && !self.password_input().is_empty() {
                    self.next();
                    HandleResult::NeedsRender
                } else {
                    HandleResult::Ignored
                }
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Default for MasterPasswordScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl WizardStepValidator for MasterPasswordScreen {
    fn validate_step(&self) -> bool {
        self.can_complete()
    }

    fn validation_error(&self) -> Option<String> {
        self.validation_error().map(|s| s.to_string())
    }

    fn sync_to_state(&self, state: &mut WizardState) {
        if let Some(pwd) = self.get_password() {
            state.set_master_password(pwd.clone());
            state.set_master_password_confirm(pwd);
        }
    }

    fn load_from_state(&mut self, state: &WizardState) {
        if let Some(pwd) = &state.master_password {
            self.set_password_input(pwd.clone());
        }
        if let Some(pwd) = &state.master_password_confirm {
            self.set_confirm_input(pwd.clone());
        }
        self.update_strength();
        self.update_match_status();
    }

    fn clear_input(&mut self) {
        self.clear();
    }
}

impl WizardStepScreen for MasterPasswordScreen {
    fn step_name(&self) -> &'static str {
        "Master Password"
    }

    fn has_changes(&self) -> bool {
        !self.password_input().is_empty() || !self.confirm_input().is_empty()
    }
}

/// RenderScreen trait implementation for rendering the screen
pub trait RenderScreen {
    fn render_screen(&self, frame: &mut Frame, area: Rect);
}

impl RenderScreen for MasterPasswordScreen {
    fn render_screen(&self, frame: &mut Frame, area: Rect) {
        render(self, frame, area);
    }
}
