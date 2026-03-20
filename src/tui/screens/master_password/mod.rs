//! Master Password Setup Screen
//!
//! Allows users to set up their device-specific master password for encrypting the Passkey.

mod impls;
mod render;
#[cfg(test)]
mod tests;
mod types;

use crate::health::strength::calculate_strength;
pub use types::PasswordStrength;

/// Master password setup screen
#[derive(Debug, Clone)]
pub struct MasterPasswordScreen {
    /// First password input
    password_input: String,
    /// Confirmation password input
    confirm_input: String,
    /// Whether showing first password field (true) or confirmation (false)
    show_first: bool,
    /// Current password strength
    strength: PasswordStrength,
    /// Validation error message
    validation_error: Option<String>,
    /// Whether passwords match
    passwords_match: bool,
}

impl MasterPasswordScreen {
    /// Create a new master password screen
    pub fn new() -> Self {
        Self {
            password_input: String::new(),
            confirm_input: String::new(),
            show_first: true,
            strength: PasswordStrength::Weak,
            validation_error: None,
            passwords_match: false,
        }
    }

    /// Get current password input
    pub fn password_input(&self) -> &str {
        &self.password_input
    }

    /// Get confirmation input
    pub fn confirm_input(&self) -> &str {
        &self.confirm_input
    }

    /// Check if showing first password field
    pub fn is_showing_first(&self) -> bool {
        self.show_first
    }

    /// Get password strength
    pub fn strength(&self) -> PasswordStrength {
        self.strength
    }

    /// Get validation error
    pub fn validation_error(&self) -> Option<&str> {
        self.validation_error.as_deref()
    }

    /// Check if passwords match
    pub fn passwords_match(&self) -> bool {
        self.passwords_match
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }

        if self.show_first {
            self.password_input.push(c);
            self.update_strength();
            self.validation_error = None;
        } else {
            self.confirm_input.push(c);
            self.update_match_status();
            self.validation_error = None;
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.show_first {
            self.password_input.pop();
            self.update_strength();
        } else {
            self.confirm_input.pop();
            self.update_match_status();
        }
        self.validation_error = None;
    }

    /// Move to confirmation field
    pub fn next(&mut self) {
        if self.show_first && !self.password_input.is_empty() {
            self.show_first = false;
        }
    }

    /// Go back to password field
    pub fn back(&mut self) {
        if !self.show_first {
            self.show_first = true;
        }
    }

    /// Check if the wizard can complete
    pub fn can_complete(&self) -> bool {
        !self.password_input.is_empty()
            && !self.confirm_input.is_empty()
            && self.passwords_match
            && self.password_input.len() >= 8
    }

    /// Get the password if valid
    pub fn get_password(&self) -> Option<String> {
        if self.can_complete() {
            Some(self.password_input.clone())
        } else {
            None
        }
    }

    /// Update password strength based on current input
    pub(super) fn update_strength(&mut self) {
        let score = calculate_strength(&self.password_input);
        self.strength = if score < 50 {
            PasswordStrength::Weak
        } else if score < 70 {
            PasswordStrength::Medium
        } else {
            PasswordStrength::Strong
        };
    }

    /// Update match status
    pub(super) fn update_match_status(&mut self) {
        self.passwords_match =
            !self.confirm_input.is_empty() && self.password_input == self.confirm_input;
    }

    /// Validate and return error if any
    pub fn validate(&self) -> Result<(), String> {
        if self.password_input.is_empty() {
            return Err("Please enter a master password".to_string());
        }

        if self.password_input.len() < 8 {
            return Err("Master password must be at least 8 characters".to_string());
        }

        if self.confirm_input.is_empty() {
            return Err("Please re-enter the master password".to_string());
        }

        if !self.passwords_match {
            return Err("Passwords do not match".to_string());
        }

        Ok(())
    }

    /// Clear all inputs
    pub fn clear(&mut self) {
        self.password_input.clear();
        self.confirm_input.clear();
        self.show_first = true;
        self.strength = PasswordStrength::Weak;
        self.validation_error = None;
        self.passwords_match = false;
    }

    /// Set password input (used by load_from_state)
    pub(super) fn set_password_input(&mut self, input: String) {
        self.password_input = input;
    }

    /// Set confirm input (used by load_from_state)
    pub(super) fn set_confirm_input(&mut self, input: String) {
        self.confirm_input = input;
    }

    /// Render the master password screen
    pub fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        render::render(self, frame, area);
    }
}

// Re-export for backward compatibility
pub use impls::RenderScreen;
