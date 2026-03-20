//! Edit Password Screen
//!
//! Form for editing an existing password entry.

mod handlers;
mod render;
mod types;

#[cfg(test)]
mod tests;

pub use types::{EditFormField, EditedPasswordFields};

use crate::tui::services::TuiCryptoService;
use crate::tui::traits::{
    AppEvent, Component, ComponentId, CryptoService, HandleResult, PasswordPolicy, PasswordType,
};
use uuid::Uuid;

/// Edit password screen
pub struct EditPasswordScreen {
    /// Password ID being edited
    password_id: Uuid,
    /// Password name (read-only, for display)
    password_name: String,

    /// Editable fields
    pub(super) username: String,
    /// New password (None = keep original)
    pub(super) new_password: Option<String>,
    /// Whether the new password is visible
    pub(super) password_visible: bool,
    pub(super) url: String,
    pub(super) notes: String,
    pub(super) tags: String,
    pub(super) group: String,

    /// Original password for reference
    original_password: String,

    /// Password generation settings
    pub(super) password_type: PasswordType,
    pub(super) password_length: u8,

    /// UI state
    pub(super) focused_field: usize,

    /// Component ID
    id: ComponentId,
}

impl EditPasswordScreen {
    /// Create an empty edit screen (placeholder)
    pub fn empty() -> Self {
        Self {
            password_id: Uuid::nil(),
            password_name: String::new(),
            username: String::new(),
            new_password: None,
            password_visible: false,
            url: String::new(),
            notes: String::new(),
            tags: String::new(),
            group: "Personal".to_string(),
            original_password: String::new(),
            password_type: PasswordType::Random,
            password_length: 16,
            focused_field: 0,
            id: ComponentId::new(4002),
        }
    }

    /// Create a new edit screen from existing password data
    #[allow(clippy::too_many_arguments)]
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
            new_password: None, // Keep original by default
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
        self.new_password
            .as_deref()
            .unwrap_or(&self.original_password)
    }

    /// Check if password was changed
    pub fn is_password_changed(&self) -> bool {
        self.new_password.is_some()
    }

    /// Get the edited password record
    pub fn get_edited_fields(&self) -> EditedPasswordFields {
        EditedPasswordFields {
            id: self.password_id,
            name: self.password_name.clone(),
            username: if self.username.is_empty() {
                None
            } else {
                Some(self.username.clone())
            },
            password: self.new_password.clone(),
            url: if self.url.is_empty() {
                None
            } else {
                Some(self.url.clone())
            },
            notes: if self.notes.is_empty() {
                None
            } else {
                Some(self.notes.clone())
            },
            tags: if self.tags.is_empty() {
                vec![]
            } else {
                self.tags.split(',').map(|s| s.trim().to_string()).collect()
            },
            group_id: if self.group.is_empty() {
                None
            } else {
                Some(self.group.clone())
            },
        }
    }

    /// Get the password name being edited
    pub fn password_name(&self) -> &str {
        &self.password_name
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
