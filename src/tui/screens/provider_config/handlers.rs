//! Key event handlers for ProviderConfigScreen
//!
//! Contains keyboard handling logic for the provider configuration form.

use super::ProviderConfigScreen;
use crate::tui::traits::{Action, HandleResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl ProviderConfigScreen {
    /// Handles a key event
    pub fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Enter => {
                // Test connection - just close for now
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save configuration - just close for now
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Tab | KeyCode::Down => {
                self.focus_next_field();
                return HandleResult::NeedsRender;
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focus_prev_field();
                return HandleResult::NeedsRender;
            }
            KeyCode::Char(c) => {
                self.handle_char_input(c);
                return HandleResult::NeedsRender;
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                return HandleResult::NeedsRender;
            }
            _ => {}
        }

        HandleResult::Ignored
    }

    /// Focuses the next field
    fn focus_next_field(&mut self) {
        if self.focused_index < self.fields.len() - 1 {
            self.fields[self.focused_index].is_focused = false;
            self.focused_index += 1;
            self.fields[self.focused_index].is_focused = true;
        }
    }

    /// Focuses the previous field
    fn focus_prev_field(&mut self) {
        if self.focused_index > 0 {
            self.fields[self.focused_index].is_focused = false;
            self.focused_index -= 1;
            self.fields[self.focused_index].is_focused = true;
        }
    }

    /// Handles character input
    fn handle_char_input(&mut self, c: char) {
        if let Some(field) = self.fields.get_mut(self.focused_index) {
            field.value.push(c);
        }
    }

    /// Handles backspace
    fn handle_backspace(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focused_index) {
            field.value.pop();
        }
    }
}
