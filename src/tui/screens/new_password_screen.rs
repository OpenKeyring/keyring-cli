//!
//! New Password Screen
//!
//! Form for creating a new password entry

use crate::tui::services::TuiCryptoService;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render, Action, PasswordPolicy, PasswordType, AppEvent, CryptoService};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::HashMap;
use uuid::Uuid;

/// Form field identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormField {
    Name,
    Username,
    PasswordType,
    PasswordLength,
    Password,
    Url,
    Notes,
    Tags,
    Group,
}

impl FormField {
    fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Username => "Username",
            Self::PasswordType => "Password Type",
            Self::PasswordLength => "Length",
            Self::Password => "Password",
            Self::Url => "URL",
            Self::Notes => "Notes",
            Self::Tags => "Tags",
            Self::Group => "Group",
        }
    }

    fn is_required(&self) -> bool {
        matches!(self, Self::Name)
    }

    fn index(&self) -> usize {
        match self {
            Self::Name => 0,
            Self::Username => 1,
            Self::PasswordType => 2,
            Self::PasswordLength => 3,
            Self::Password => 4,
            Self::Url => 5,
            Self::Notes => 6,
            Self::Tags => 7,
            Self::Group => 8,
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Name),
            1 => Some(Self::Username),
            2 => Some(Self::PasswordType),
            3 => Some(Self::PasswordLength),
            4 => Some(Self::Password),
            5 => Some(Self::Url),
            6 => Some(Self::Notes),
            7 => Some(Self::Tags),
            8 => Some(Self::Group),
            _ => None,
        }
    }
}

/// New password screen
pub struct NewPasswordScreen {
    /// Form fields
    name: String,
    username: String,
    password: String,
    password_visible: bool,
    url: String,
    notes: String,
    tags: String,
    group: String,

    /// Password generation settings
    password_type: PasswordType,
    password_length: u8,

    /// UI state
    focused_field: usize,
    input_position: usize,

    /// Validation errors
    errors: HashMap<FormField, String>,

    /// Component ID
    id: ComponentId,
}

impl NewPasswordScreen {
    /// Create a new screen
    pub fn new() -> Self {
        let mut screen = Self {
            name: String::new(),
            username: String::new(),
            password: String::new(),
            password_visible: false,
            url: String::new(),
            notes: String::new(),
            tags: String::new(),
            group: "Personal".to_string(),
            password_type: PasswordType::Random,
            password_length: 16,
            focused_field: 0,
            input_position: 0,
            errors: HashMap::new(),
            id: ComponentId::new(4001),
        };
        // Generate initial password
        screen.generate_password();
        screen
    }

    /// Generate password based on current settings
    pub fn generate_password(&mut self) {
        let service = TuiCryptoService::new();
        let policy = PasswordPolicy {
            length: self.password_length,
            min_digits: 2,
            min_special: 1,
            min_lowercase: 1,
            min_uppercase: 1,
            password_type: self.password_type,
        };
        if let Ok(pwd) = service.generate_password(&policy) {
            self.password = pwd;
        }
    }

    /// Validate the form
    pub fn validate(&self) -> Result<(), Vec<(FormField, String)>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push((FormField::Name, "Name is required".to_string()));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the created password record (for mock implementation)
    pub fn get_password_record(&self) -> Option<NewPasswordRecord> {
        if self.validate().is_err() {
            return None;
        }

        Some(NewPasswordRecord {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            username: if self.username.is_empty() { None } else { Some(self.username.clone()) },
            password: self.password.clone(),
            url: if self.url.is_empty() { None } else { Some(self.url.clone()) },
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            tags: if self.tags.is_empty() {
                vec![]
            } else {
                self.tags.split(',').map(|s| s.trim().to_string()).collect()
            },
            group: self.group.clone(),
        })
    }
}

impl Default for NewPasswordScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Password record created from the form
#[derive(Debug, Clone)]
pub struct NewPasswordRecord {
    pub id: Uuid,
    pub name: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub group: String,
}

impl Render for NewPasswordScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("  New Password  ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        block.render(area, buf);

        let field_count = 9;
        let row_height = 3;
        let start_y = inner.y + 1;

        // Render each field
        for i in 0..field_count {
            let y = start_y + (i as u16) * row_height;
            if y >= inner.y + inner.height {
                break;
            }

            let field = match FormField::from_index(i) {
                Some(f) => f,
                None => continue,
            };

            let is_focused = i == self.focused_field;
            let error = self.errors.get(&field);

            // Field label
            let label = if field.is_required() {
                format!("{}*:", field.label())
            } else {
                format!("{}:", field.label())
            };

            let label_style = if is_focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let _field_width = inner.width - 4;

            // Render based on field type
            match field {
                FormField::Name => {
                    let content = if self.name.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.name)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);
                }
                FormField::Username => {
                    let content = if self.username.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.username)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);
                }
                FormField::PasswordType => {
                    let type_label = self.password_type.label();
                    let display = format!("[{}]  ", type_label);
                    buf.set_string(inner.x + 2, y, &label, label_style);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
                FormField::PasswordLength => {
                    let display = format!("[{}]  ", self.password_length);
                    buf.set_string(inner.x + 2, y, &label, label_style);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
                FormField::Password => {
                    let display = if self.password_visible {
                        Span::raw(&self.password)
                    } else {
                        Span::raw("•".repeat(self.password.len().max(16)))
                    };
                    let input = Paragraph::new(display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);

                    // Show regenerate hint
                    let hint = "[r] Regenerate  [Space] Show/Hide";
                    buf.set_string(inner.x + 20, y + 1, hint,
                        Style::default().fg(Color::DarkGray));
                }
                FormField::Url => {
                    let content = if self.url.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.url)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);
                }
                FormField::Notes => {
                    buf.set_string(inner.x + 2, y, &label, label_style);
                    // Notes takes 2 rows
                    let notes_display = if self.notes.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.notes)
                    };
                    let input = Paragraph::new(notes_display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 3), buf);
                }
                FormField::Tags => {
                    let content = if self.tags.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.tags)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE))
                        .wrap(ratatui::widgets::Wrap { trim: false });

                    buf.set_string(inner.x + 2, y, &label, label_style);
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);

                    let hint = "(comma separated)";
                    buf.set_string(inner.x + 20, y + 1, hint, Style::default().fg(Color::DarkGray));
                }
                FormField::Group => {
                    let display = format!("[{}]", self.group);
                    buf.set_string(inner.x + 2, y, &label, label_style);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
            }

            // Show error if any
            if let Some(err) = error {
                buf.set_string(inner.x + 2, y + row_height - 1, err,
                    Style::default().fg(Color::Red));
            }
        }

        // Help text at bottom
        let help_y = inner.y + inner.height - 2;
        let help = "[Tab] Next  [Esc] Cancel  [Enter] Save";
        buf.set_string(inner.x + 2, help_y, help, Style::default().fg(Color::DarkGray));

        // Show validation errors at bottom
        if !self.errors.is_empty() {
            let error_y = inner.y + inner.height - 4;
            for (i, (field, err)) in self.errors.iter().enumerate() {
                let msg = format!("{}: {}", field.label(), err);
                buf.set_string(inner.x + 2, error_y + i as u16, msg,
                    Style::default().fg(Color::Red));
            }
        }
    }
}

impl Interactive for NewPasswordScreen {
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
                self.focused_field = if self.focused_field == 0 {
                    8
                } else {
                    self.focused_field - 1
                };
                return HandleResult::NeedsRender;
            }
            KeyCode::Up => {
                self.focused_field = if self.focused_field == 0 {
                    8
                } else {
                    self.focused_field - 1
                };
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
                let field = FormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        FormField::PasswordType => {
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
                        FormField::Group => {
                            // Toggle group (simplified)
                        }
                        _ => {}
                    }
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Right => {
                let field = FormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        FormField::PasswordType => {
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
                        FormField::Group => {
                            // Toggle group (simplified)
                        }
                        _ => {}
                    }
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Backspace => {
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
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Char(c) => {
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
                    return HandleResult::NeedsRender;
                }
            }
            _ => {}
        }

        HandleResult::Ignored
    }
}

impl Component for NewPasswordScreen {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_event(&mut self, _event: &AppEvent) -> HandleResult {
        HandleResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_password_screen_creation() {
        let screen = NewPasswordScreen::new();
        assert!(!screen.password.is_empty());
    }

    #[test]
    fn test_validation_empty_name() {
        let screen = NewPasswordScreen::new();
        let result = screen.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_with_name() {
        let mut screen = NewPasswordScreen::new();
        screen.name = "Test Password".to_string();
        let result = screen.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_generation() {
        let mut screen = NewPasswordScreen::new();
        screen.name = "Test".to_string();
        screen.password_length = 20;
        screen.generate_password();
        assert_eq!(screen.password.len(), 20);
    }

    #[test]
    fn test_password_toggle_visibility() {
        let mut screen = NewPasswordScreen::new();
        assert!(!screen.password_visible);
        screen.password_visible = true;
        assert!(screen.password_visible);
    }

    #[test]
    fn test_get_password_record() {
        let mut screen = NewPasswordScreen::new();
        screen.name = "Test Password".to_string();
        let record = screen.get_password_record();
        assert!(record.is_some());
        let rec = record.unwrap();
        assert_eq!(rec.name, "Test Password");
    }
}