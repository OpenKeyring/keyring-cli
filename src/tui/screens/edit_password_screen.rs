//!
//! Edit Password Screen
//!
//! Form for editing an existing password entry

use crate::tui::services::TuiCryptoService;
use crate::tui::traits::{Component, ComponentId, HandleResult, Interactive, Render, PasswordPolicy, PasswordType, AppEvent, CryptoService};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget},
};
use uuid::Uuid;

/// Form field identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditFormField {
    Username,
    PasswordType,
    PasswordLength,
    Password,
    Url,
    Notes,
    Tags,
    Group,
}

impl EditFormField {
    fn label(&self) -> &'static str {
        match self {
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

    fn index(&self) -> usize {
        match self {
            Self::Username => 0,
            Self::PasswordType => 1,
            Self::PasswordLength => 2,
            Self::Password => 3,
            Self::Url => 4,
            Self::Notes => 5,
            Self::Tags => 6,
            Self::Group => 7,
        }
    }

    fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Username),
            1 => Some(Self::PasswordType),
            2 => Some(Self::PasswordLength),
            3 => Some(Self::Password),
            4 => Some(Self::Url),
            5 => Some(Self::Notes),
            6 => Some(Self::Tags),
            7 => Some(Self::Group),
            _ => None,
        }
    }
}

/// Edit password screen
pub struct EditPasswordScreen {
    /// Password ID being edited
    password_id: Uuid,
    /// Password name (read-only, for display)
    password_name: String,

    /// Editable fields
    username: String,
    /// New password (None = keep original)
    new_password: Option<String>,
    /// Whether the new password is visible
    password_visible: bool,
    url: String,
    notes: String,
    tags: String,
    group: String,

    /// Original password for reference
    original_password: String,

    /// Password generation settings
    password_type: PasswordType,
    password_length: u8,

    /// UI state
    focused_field: usize,

    /// Component ID
    id: ComponentId,
}

impl EditPasswordScreen {
    /// Create a new edit screen from existing password data
    pub fn new(
        id: Uuid,
        name: &str,
        username: Option<&str>,
        password: &str,
        url: Option<&str>,
        notes: Option<&str>,
        tags: &[String],
        group: Option<&str>,
    ) -> Self {
        Self {
            password_id: id,
            password_name: name.to_string(),
            username: username.unwrap_or("").to_string(),
            new_password: None,  // Keep original by default
            password_visible: false,
            url: url.unwrap_or("").to_string(),
            notes: notes.unwrap_or("").to_string(),
            tags: tags.join(", "),
            group: group.unwrap_or("Personal").to_string(),
            original_password: password.to_string(),
            password_type: PasswordType::Random,
            password_length: 16,
            focused_field: 0,
            id: ComponentId::new(4002),
        }
    }

    /// Generate a new password
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
            self.new_password = Some(pwd);
        }
    }

    /// Get the current password (either new or original)
    pub fn get_current_password(&self) -> &str {
        self.new_password.as_deref().unwrap_or(&self.original_password)
    }

    /// Check if password was changed
    pub fn is_password_changed(&self) -> bool {
        self.new_password.is_some()
    }

    /// Get the edited password record
    pub fn get_edited_fields(&self) -> EditedPasswordFields {
        EditedPasswordFields {
            id: self.password_id,
            username: if self.username.is_empty() { None } else { Some(self.username.clone()) },
            password: self.new_password.clone(),
            url: if self.url.is_empty() { None } else { Some(self.url.clone()) },
            notes: if self.notes.is_empty() { None } else { Some(self.notes.clone()) },
            tags: if self.tags.is_empty() {
                vec![]
            } else {
                self.tags.split(',').map(|s| s.trim().to_string()).collect()
            },
            group: self.group.clone(),
        }
    }
}

/// Edited password fields result
#[derive(Debug, Clone)]
pub struct EditedPasswordFields {
    pub id: Uuid,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub group: String,
}

impl Render for EditPasswordScreen {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("  Edit Password  ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner = block.inner(area);
        block.render(area, buf);

        // Display name (read-only) at the top
        let name_y = inner.y + 1;
        let name_style = Style::default().fg(Color::DarkGray);
        buf.set_string(inner.x + 2, name_y, &format!("Name: {} (read-only)", self.password_name), name_style);

        let start_y = inner.y + 3;
        let row_height = 3;

        // Render each editable field
        for i in 0..8 {
            let y = start_y + (i as u16) * row_height;
            if y >= inner.y + inner.height {
                break;
            }

            let field = match EditFormField::from_index(i) {
                Some(f) => f,
                None => continue,
            };

            let is_focused = i == self.focused_field;
            let label_style = if is_focused {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Field label
            let label = format!("{}:", field.label());
            buf.set_string(inner.x + 2, y, &label, label_style);

            // Render field content
            match field {
                EditFormField::Username => {
                    let content = if self.username.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.username)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);
                }
                EditFormField::PasswordType => {
                    let type_label = self.password_type.label();
                    let display = format!("[{}]  ", type_label);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
                EditFormField::PasswordLength => {
                    let display = format!("[{}]  ", self.password_length);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
                EditFormField::Password => {
                    let display = if self.password_visible {
                        Span::raw(self.get_current_password())
                    } else {
                        Span::raw("•".repeat(self.get_current_password().len().max(16)))
                    };
                    let input = Paragraph::new(display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));

                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);

                    // Show hints
                    let hint = if self.new_password.is_some() {
                        "[r] Regenerate  [Space] Show/Hide  (modified)"
                    } else {
                        "[r] Regenerate  [Space] Show/Hide  (original)"
                    };
                    buf.set_string(inner.x + 20, y + 1, hint,
                        Style::default().fg(if self.new_password.is_some() { Color::Yellow } else { Color::DarkGray }));
                }
                EditFormField::Url => {
                    let content = if self.url.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.url)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);
                }
                EditFormField::Notes => {
                    let notes_display = if self.notes.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.notes)
                    };
                    let input = Paragraph::new(notes_display)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 3), buf);
                }
                EditFormField::Tags => {
                    let content = if self.tags.is_empty() {
                        Span::raw("")
                    } else {
                        Span::raw(&self.tags)
                    };
                    let input = Paragraph::new(content)
                        .style(label_style)
                        .block(Block::default().borders(Borders::NONE));
                    input.render(Rect::new(inner.x + 2, y + 1, inner.x + inner.width - 2, y + 2), buf);

                    let hint = "(comma separated)";
                    buf.set_string(inner.x + 20, y + 1, hint, Style::default().fg(Color::DarkGray));
                }
                EditFormField::Group => {
                    let display = format!("[{}]", self.group);
                    buf.set_string(inner.x + 2, y + 1, &display,
                        Style::default().fg(if is_focused { Color::Yellow } else { Color::White }));
                }
            }
        }

        // Help text at bottom
        let help_y = inner.y + inner.height - 2;
        let help = "[Tab] Next  [Esc] Cancel  [Enter] Save";
        buf.set_string(inner.x + 2, help_y, help, Style::default().fg(Color::DarkGray));
    }
}

impl Interactive for EditPasswordScreen {
    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if key.kind == KeyEventKind::Release {
            return HandleResult::Ignored;
        }

        match key.code {
            KeyCode::Esc => {
                return HandleResult::Action(crate::tui::traits::Action::CloseScreen);
            }
            KeyCode::Enter => {
                return HandleResult::Action(crate::tui::traits::Action::CloseScreen);
            }
            KeyCode::Tab => {
                self.focused_field = (self.focused_field + 1) % 8;
                return HandleResult::NeedsRender;
            }
            KeyCode::BackTab => {
                self.focused_field = if self.focused_field == 0 { 7 } else { self.focused_field - 1 };
                return HandleResult::NeedsRender;
            }
            KeyCode::Up => {
                self.focused_field = if self.focused_field == 0 { 7 } else { self.focused_field - 1 };
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
                let field = EditFormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        EditFormField::PasswordType => {
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
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Right => {
                let field = EditFormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        EditFormField::PasswordType => {
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
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Backspace => {
                let field = EditFormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        EditFormField::Username => { self.username.pop(); }
                        EditFormField::Url => { self.url.pop(); }
                        EditFormField::Notes => { self.notes.pop(); }
                        EditFormField::Tags => { self.tags.pop(); }
                        _ => {}
                    }
                    return HandleResult::NeedsRender;
                }
            }
            KeyCode::Char(c) => {
                let field = EditFormField::from_index(self.focused_field);
                if let Some(f) = field {
                    match f {
                        EditFormField::Username => { self.username.push(c); }
                        EditFormField::Url => { self.url.push(c); }
                        EditFormField::Notes => { self.notes.push(c); }
                        EditFormField::Tags => { self.tags.push(c); }
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

impl Component for EditPasswordScreen {
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
    use uuid::Uuid;

    #[test]
    fn test_edit_password_screen_creation() {
        let id = Uuid::new_v4();
        let screen = EditPasswordScreen::new(
            id,
            "Test Password",
            Some("user@example.com"),
            "original_password",
            Some("https://example.com"),
            Some("Test notes"),
            &["tag1".to_string(), "tag2".to_string()],
            Some("Personal"),
        );

        assert_eq!(screen.password_name, "Test Password");
        assert_eq!(screen.username, "user@example.com");
        assert!(screen.new_password.is_none()); // Should keep original
        assert_eq!(screen.url, "https://example.com");
    }

    #[test]
    fn test_password_regeneration() {
        let id = Uuid::new_v4();
        let mut screen = EditPasswordScreen::new(
            id,
            "Test",
            None,
            "original",
            None,
            None,
            &[],
            None,
        );

        assert!(screen.new_password.is_none());
        screen.generate_password();
        assert!(screen.new_password.is_some());
        assert!(screen.is_password_changed());
    }

    #[test]
    fn test_get_current_password() {
        let id = Uuid::new_v4();
        let mut screen = EditPasswordScreen::new(
            id,
            "Test",
            None,
            "original_password",
            None,
            None,
            &[],
            None,
        );

        // Initially returns original
        assert_eq!(screen.get_current_password(), "original_password");

        // After generation, returns new
        screen.generate_password();
        assert_ne!(screen.get_current_password(), "original_password");
    }

    #[test]
    fn test_get_edited_fields() {
        let id = Uuid::new_v4();
        let screen = EditPasswordScreen::new(
            id,
            "Test",
            Some("user"),
            "pass",
            Some("https://example.com"),
            Some("notes"),
            &["tag1".to_string()],
            Some("Work"),
        );

        let fields = screen.get_edited_fields();
        assert_eq!(fields.id, id);
        assert_eq!(fields.username, Some("user".to_string()));
        assert_eq!(fields.url, Some("https://example.com".to_string()));
        assert_eq!(fields.notes, Some("notes".to_string()));
        assert_eq!(fields.group, "Work");
    }

    #[test]
    fn test_toggle_password_visibility() {
        let id = Uuid::new_v4();
        let mut screen = EditPasswordScreen::new(
            id,
            "Test",
            None,
            "password123",
            None,
            None,
            &[],
            None,
        );

        assert!(!screen.password_visible);

        // Simulate Space key on password field
        screen.focused_field = EditFormField::Password.index();
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
        let result = screen.handle_key(key);
        assert!(matches!(result, HandleResult::NeedsRender));
        assert!(screen.password_visible);
    }
}
