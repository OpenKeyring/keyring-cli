//! New Password Screen
//!
//! Form for creating a new password entry.

mod handlers;
mod render;
mod types;

#[cfg(test)]
mod tests;

pub use types::{FormField, NewPasswordRecord};

use crate::tui::services::TuiCryptoService;
use crate::tui::traits::{
    AppEvent, Component, ComponentId, CryptoService, HandleResult, PasswordPolicy, PasswordType,
};
use std::collections::HashMap;

/// New password screen
pub struct NewPasswordScreen {
    /// Form fields
    pub(super) name: String,
    pub(super) username: String,
    pub(super) password: String,
    pub(super) password_visible: bool,
    pub(super) url: String,
    pub(super) notes: String,
    pub(super) tags: String,
    pub(super) group: String,

    /// Password generation settings
    pub(super) password_type: PasswordType,
    pub(super) password_length: u8,

    /// UI state
    pub(super) focused_field: usize,
    pub(super) input_position: usize,

    /// Validation errors
    pub(super) errors: HashMap<FormField, String>,

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
            id: uuid::Uuid::new_v4(),
            name: self.name.clone(),
            username: if self.username.is_empty() {
                None
            } else {
                Some(self.username.clone())
            },
            password: self.password.clone(),
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
            group: self.group.clone(),
        })
    }
}

impl Default for NewPasswordScreen {
    fn default() -> Self {
        Self::new()
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
