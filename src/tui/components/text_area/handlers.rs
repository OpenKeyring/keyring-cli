//! Trait implementations for TextArea
//!
//! Contains Interactive and Component trait implementations.

use super::TextArea;
use crate::tui::error::TuiResult;
use crate::tui::traits::{Component, HandleResult, Interactive, ValidationTrigger};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

impl Interactive for TextArea {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        // Only handle press events
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Char(ch) => {
                self.insert_char(ch);
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                self.insert_newline();
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                self.backspace();
                HandleResult::Consumed
            }
            KeyCode::Delete => {
                self.delete();
                HandleResult::Consumed
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Left: move to line start
                    self.move_home();
                } else {
                    self.move_left();
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Right: move to line end
                    self.move_end();
                } else {
                    self.move_right();
                }
                HandleResult::Consumed
            }
            KeyCode::Up => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Up: scroll up
                    self.scroll_up();
                } else {
                    self.move_up();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Down: scroll down
                    self.scroll_down();
                } else {
                    self.move_down();
                }
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.move_home();
                HandleResult::Consumed
            }
            KeyCode::End => {
                self.move_end();
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                self.page_up();
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                self.page_down();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Component for TextArea {
    fn id(&self) -> crate::tui::traits::ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus_gain(&mut self) -> TuiResult<()> {
        self.focused = true;
        // Validate current value
        if self.validation.is_some() {
            self.validation_result = Some(self.validate());
        }
        Ok(())
    }

    fn on_focus_loss(&mut self) -> TuiResult<()> {
        self.focused = false;
        // Validate on blur if configured
        if let Some(ref validation) = self.validation {
            if matches!(validation.trigger, ValidationTrigger::OnBlur) {
                self.validation_result = Some(self.validate());
            }
        }
        Ok(())
    }

    fn on_mount(&mut self) -> TuiResult<()> {
        // Initial validation
        if self.validation.is_some() {
            self.validation_result = Some(self.validate());
        }
        Ok(())
    }

    fn before_render(&mut self) -> TuiResult<()> {
        // Scroll position will be updated during render
        Ok(())
    }
}
