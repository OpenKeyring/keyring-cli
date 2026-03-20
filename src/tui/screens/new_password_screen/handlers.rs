//! Key event handlers for NewPasswordScreen
//!
//! Contains keyboard handling logic for the new password form.

use super::{FormField, NewPasswordScreen};
use crate::tui::traits::{Action, HandleResult};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

impl crate::tui::traits::Interactive for NewPasswordScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                return HandleResult::Action(Action::CloseScreen);
            }
            KeyCode::Enter => {
                // Validate and save
                if self.validate().is_ok() {
                    return HandleResult::Action(Action::CloseScreen);
                } else {
                    // Show errors
                    let _ = self.validate();
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Tab => {
                self.focused_field = (self.focused_field + 1) % 9;
                return HandleResult::NeedsRender;
            }
            KeyCode::BackTab => {
                self.focused_field = if self.focused_field == 0 { 8 } else { self.focused_field - 1 };
                return HandleResult::NeedsRender;
            }
            KeyCode::Up => {
                self.focused_field = if self.focused_field == 0 { 8 } else { self.focused_field - 1 };
                return HandleResult::NeedsRender;
            }
            KeyCode::Down => {
                self.focused_field = (self.focused_field + 1) % 9;
                return HandleResult::NeedsRender;
            }
            KeyCode::Char(' ') => {
                if self.focused_field == FormField::Password.index() {
                    self.password_visible = !self.password_visible;
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Char('r') => {
                if self.focused_field == FormField::Password.index() {
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

impl NewPasswordScreen {
    /// Handle left arrow key
    fn handle_left_key(&mut self) {
        let field = FormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                FormField::PasswordType => {
                    use crate::tui::traits::PasswordType;
                    self.password_type = match self.password_type {
                        PasswordType::Random => PasswordType::Pin,
                        PasswordType::Memorable => PasswordType::Random,
                        PasswordType::Pin => PasswordType::Memorable,
                    };
                    self.generate_password();
                }
                FormField::PasswordLength => {
                    if self.password_length > 8 {
                        self.password_length -= 1;
                        self.generate_password();
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle right arrow key
    fn handle_right_key(&mut self) {
        let field = FormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                FormField::PasswordType => {
                    use crate::tui::traits::PasswordType;
                    self.password_type = match self.password_type {
                        PasswordType::Random => PasswordType::Memorable,
                        PasswordType::Memorable => PasswordType::Pin,
                        PasswordType::Pin => PasswordType::Random,
                    };
                    self.generate_password();
                }
                FormField::PasswordLength => {
                    if self.password_length < 64 {
                        self.password_length += 1;
                        self.generate_password();
                    }
                }
                _ => {}
            }
        }
    }

    /// Handle backspace key
    fn handle_backspace(&mut self) {
        let field = FormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                FormField::Name => {
                    self.name.pop();
                    self.errors.remove(&FormField::Name);
                }
                FormField::Username => {
                    self.username.pop();
                }
                FormField::Password => {
                    self.password.pop();
                }
                FormField::Url => {
                    self.url.pop();
                }
                FormField::Notes => {
                    self.notes.pop();
                }
                FormField::Tags => {
                    self.tags.pop();
                }
                _ => {}
            }
        }
    }

    /// Handle character input
    fn handle_char_input(&mut self, c: char) {
        let field = FormField::from_index(self.focused_field);
        if let Some(f) = field {
            match f {
                FormField::Name => {
                    self.name.push(c);
                    self.errors.remove(&FormField::Name);
                }
                FormField::Username => {
                    self.username.push(c);
                }
                FormField::Password => {
                    self.password.push(c);
                }
                FormField::Url => {
                    self.url.push(c);
                }
                FormField::Notes => {
                    self.notes.push(c);
                }
                FormField::Tags => {
                    self.tags.push(c);
                }
                _ => {}
            }
        }
    }
}
