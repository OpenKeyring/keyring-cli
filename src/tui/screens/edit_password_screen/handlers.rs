//! Key event handlers for EditPasswordScreen
//!
//! Contains keyboard handling logic for the edit password form.

use super::{EditFormField, EditPasswordScreen};
use crate::tui::traits::{Action, HandleResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

impl crate::tui::traits::Interactive for EditPasswordScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Enter => {
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Tab => {
                self.focused_field = (self.focused_field + 1) % 8;
                return HandleResult::NeedsRender;
            }
            KeyCode::BackTab => {
                self.focused_field = if self.focused_field == 0 {
                    7
                } else {
                    self.focused_field - 1
                };
                return HandleResult::NeedsRender;
            }
            KeyCode::Up => {
                self.focused_field = if self.focused_field == 0 {
                    7
                } else {
                    self.focused_field - 1
                };
                return HandleResult::NeedsRender;
            }
            KeyCode::Down => {
                self.focused_field = (self.focused_field + 1) % 8;
                return HandleResult::NeedsRender;
            }
            KeyCode::Char(' ') => {
                if self.focused_field == EditFormField::Password.index() {
                    self.password_visible = !self.password_visible;
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Char('r') => {
                if self.focused_field == EditFormField::Password.index() {
                    self.generate_password();
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Left => {
                self.handle_left_key();
                return HandleResult::NeedsRender;
            }
            KeyCode::Right => {
                self.handle_right_key();
                return HandleResult::NeedsRender;
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                return HandleResult::NeedsRender;
            }
            KeyCode::Char(c) => {
                self.handle_char_input(c);
                return HandleResult::NeedsRender;
            }
            _ => {}
        }

        HandleResult::Ignored
    }
}

impl EditPasswordScreen {
    /// Handle left arrow key
    fn handle_left_key(&mut self) {
        let field = EditFormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                EditFormField::PasswordType => {
                    use crate::tui::traits::PasswordType;
                    self.password_type = match self.password_type {
                        PasswordType::Random => PasswordType::Pin,
                        PasswordType::Memorable => PasswordType::Random,
                        PasswordType::Pin => PasswordType::Memorable,
                    };
                }
                EditFormField::PasswordLength => {
                    if self.password_length > 8 {
                        self.password_length -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle right arrow key
    fn handle_right_key(&mut self) {
        let field = EditFormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                EditFormField::PasswordType => {
                    use crate::tui::traits::PasswordType;
                    self.password_type = match self.password_type {
                        PasswordType::Random => PasswordType::Memorable,
                        PasswordType::Memorable => PasswordType::Pin,
                        PasswordType::Pin => PasswordType::Random,
                    };
                }
                EditFormField::PasswordLength => {
                    if self.password_length < 64 {
                        self.password_length += 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle backspace key
    fn handle_backspace(&mut self) {
        let field = EditFormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                EditFormField::Username => {
                    self.username.pop();
                }
                EditFormField::Url => {
                    self.url.pop();
                }
                EditFormField::Notes => {
                    self.notes.pop();
                }
                EditFormField::Tags => {
                    self.tags.pop();
                }
                _ => {}
            }
        }
    }

    /// Handle character input
    fn handle_char_input(&mut self, c: char) {
        let field = EditFormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                EditFormField::Username => {
                    self.username.push(c);
                }
                EditFormField::Url => {
                    self.url.push(c);
                }
                EditFormField::Notes => {
                    self.notes.push(c);
                }
                EditFormField::Tags => {
                    self.tags.push(c);
                }
                _ => {}
            }
        }
    }
}
