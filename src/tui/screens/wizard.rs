//! Wizard State Management
//!
//! Core state machine for the onboarding wizard, managing the flow between
//! different wizard steps and collecting user data.

use crate::tui::screens::welcome::WelcomeChoice;
use std::path::PathBuf;

/// Current step in the onboarding wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    // === Entry Point ===
    /// Welcome screen - choose generation or import
    Welcome,

    // === New Setup Flow ===
    /// Master password setup (first input)
    MasterPassword,
    /// Master password confirmation (re-enter)
    MasterPasswordConfirm,
    /// Security notice about password recovery
    SecurityNotice,
    /// Passkey generation screen (24-word display)
    PasskeyGenerate,
    /// Passkey verification (random 3 positions)
    PasskeyVerify,

    // === Import Flow ===
    /// Passkey import screen
    PasskeyImport,
    /// Master password setup after import
    MasterPasswordImport,
    /// Master password confirmation after import
    MasterPasswordImportConfirm,
    /// Password hint/tips screen
    PasswordHint,

    // === Common Configuration Steps ===
    /// Password generation policy configuration
    PasswordPolicy,
    /// Clipboard auto-clear timeout configuration
    ClipboardTimeout,
    /// Trash retention days configuration
    TrashRetention,
    /// Import existing passwords (optional)
    ImportPasswords,

    // === Completion ===
    /// Wizard complete
    Complete,
}

impl WizardStep {
    /// Get display name for this step
    pub fn name(&self) -> &'static str {
        match self {
            WizardStep::Welcome => "Welcome",
            WizardStep::MasterPassword => "Master Password",
            WizardStep::MasterPasswordConfirm => "Confirm Password",
            WizardStep::SecurityNotice => "Security Notice",
            WizardStep::PasskeyGenerate => "Generate PassKey",
            WizardStep::PasskeyVerify => "Verify PassKey",
            WizardStep::PasskeyImport => "Import PassKey",
            WizardStep::MasterPasswordImport => "Set Master Password",
            WizardStep::MasterPasswordImportConfirm => "Confirm Password",
            WizardStep::PasswordHint => "Password Hint",
            WizardStep::PasswordPolicy => "Password Policy",
            WizardStep::ClipboardTimeout => "Clipboard Timeout",
            WizardStep::TrashRetention => "Trash Retention",
            WizardStep::ImportPasswords => "Import Passwords",
            WizardStep::Complete => "Complete",
        }
    }
}

/// Complete state for the onboarding wizard
#[derive(Debug, Clone)]
pub struct WizardState {
    /// Current step in the wizard
    pub step: WizardStep,
    /// User's choice for Passkey setup
    pub passkey_choice: Option<WelcomeChoice>,
    /// The generated or imported Passkey words
    pub passkey_words: Option<Vec<String>>,
    /// Master password input
    pub master_password: Option<String>,
    /// Whether user confirmed they saved the Passkey
    pub confirmed: bool,
    /// Keystore path for initialization
    pub keystore_path: Option<PathBuf>,
    /// Any error message to display
    pub error: Option<String>,
}

impl WizardState {
    /// Create a new wizard state
    pub fn new() -> Self {
        Self {
            step: WizardStep::Welcome,
            passkey_choice: None,
            passkey_words: None,
            master_password: None,
            confirmed: false,
            keystore_path: None,
            error: None,
        }
    }

    /// Set the keystore path
    pub fn with_keystore_path(mut self, path: PathBuf) -> Self {
        self.keystore_path = Some(path);
        self
    }

    /// Advance to the next step
    pub fn next(&mut self) {
        self.step = match self.step {
            WizardStep::Welcome => {
                // Move to generate or import based on choice
                if let Some(WelcomeChoice::GenerateNew) = self.passkey_choice {
                    WizardStep::PasskeyGenerate
                } else {
                    WizardStep::PasskeyImport
                }
            }
            WizardStep::PasskeyGenerate => {
                // Only proceed if words are set
                if self.passkey_words.is_some() {
                    WizardStep::PasskeyConfirm
                } else {
                    // Stay on generate screen
                    WizardStep::PasskeyGenerate
                }
            }
            WizardStep::PasskeyImport => {
                // Only proceed if words are validated
                if self.passkey_words.is_some() {
                    WizardStep::MasterPassword
                } else {
                    // Stay on import screen
                    WizardStep::PasskeyImport
                }
            }
            WizardStep::PasskeyConfirm => {
                // Only proceed if confirmed
                if self.confirmed {
                    WizardStep::MasterPassword
                } else {
                    // Stay on confirmation screen
                    WizardStep::PasskeyConfirm
                }
            }
            WizardStep::MasterPassword => {
                // Proceed if password is set and valid
                if self.can_proceed() {
                    WizardStep::Complete
                } else {
                    // Stay on password screen
                    WizardStep::MasterPassword
                }
            }
            WizardStep::Complete => WizardStep::Complete, // Stay on complete
        };
    }

    /// Go back to the previous step
    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Welcome => WizardStep::Welcome, // Already at start
            WizardStep::PasskeyGenerate => WizardStep::Welcome,
            WizardStep::PasskeyImport => WizardStep::Welcome,
            WizardStep::PasskeyConfirm => {
                // If came from import, go to import, otherwise to generate
                if let Some(WelcomeChoice::ImportExisting) = self.passkey_choice {
                    WizardStep::PasskeyImport
                } else {
                    WizardStep::PasskeyGenerate
                }
            }
            WizardStep::MasterPassword => {
                // If came from import, go to import, otherwise to confirm
                if let Some(WelcomeChoice::ImportExisting) = self.passkey_choice {
                    WizardStep::PasskeyImport
                } else {
                    WizardStep::PasskeyConfirm
                }
            }
            WizardStep::Complete => WizardStep::MasterPassword,
        };
    }

    /// Check if we can proceed to the next step
    pub fn can_proceed(&self) -> bool {
        match self.step {
            WizardStep::Welcome => self.passkey_choice.is_some(),
            WizardStep::PasskeyConfirm => self.confirmed,
            WizardStep::MasterPassword => {
                self.master_password.is_some()
                    && self
                        .master_password
                        .as_ref()
                        .map(|p| p.len() >= 8)
                        .unwrap_or(false)
            }
            WizardStep::Complete => true,
            WizardStep::PasskeyGenerate => {
                // Can proceed after generating words
                self.passkey_words.is_some()
            }
            WizardStep::PasskeyImport => {
                // Can proceed after validation
                self.passkey_words.is_some()
            }
        }
    }

    /// Check if we can go back from current step
    pub fn can_go_back(&self) -> bool {
        !matches!(self.step, WizardStep::Welcome)
    }

    /// Set the passkey choice
    pub fn set_passkey_choice(&mut self, choice: WelcomeChoice) {
        self.passkey_choice = Some(choice);
    }

    /// Set the passkey words
    pub fn set_passkey_words(&mut self, words: Vec<String>) {
        self.passkey_words = Some(words);
    }

    /// Set the master password
    pub fn set_master_password(&mut self, password: String) {
        self.master_password = Some(password);
    }

    /// Set the confirmed state
    pub fn set_confirmed(&mut self, confirmed: bool) {
        self.confirmed = confirmed;
    }

    /// Toggle the confirmed state
    pub fn toggle_confirmed(&mut self) {
        self.confirmed = !self.confirmed;
    }

    /// Set an error message
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Clear any error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Check if wizard is complete
    pub fn is_complete(&self) -> bool {
        self.step == WizardStep::Complete
            && self.passkey_choice.is_some()
            && self.passkey_words.is_some()
            && self.master_password.is_some()
            && self
                .master_password
                .as_ref()
                .map(|p| p.len() >= 8)
                .unwrap_or(false)
    }

    /// Get the passkey choice
    pub fn require_passkey_choice(&self) -> Option<WelcomeChoice> {
        self.passkey_choice
    }

    /// Get the passkey words
    pub fn require_passkey_words(&self) -> Option<&[String]> {
        self.passkey_words.as_deref()
    }

    /// Get the master password
    pub fn require_master_password(&self) -> Option<&str> {
        self.master_password.as_deref()
    }

    /// Get the keystore path
    pub fn require_keystore_path(&self) -> Option<&PathBuf> {
        self.keystore_path.as_ref()
    }

    /// Reset the wizard state (useful for retry)
    pub fn reset(&mut self) {
        self.step = WizardStep::Welcome;
        self.passkey_choice = None;
        self.passkey_words = None;
        self.master_password = None;
        self.confirmed = false;
        self.error = None;
    }
}

impl Default for WizardState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_step_name() {
        assert_eq!(WizardStep::Welcome.name(), "Welcome");
        assert_eq!(WizardStep::PasskeyGenerate.name(), "Generate Passkey");
        assert_eq!(WizardStep::PasskeyImport.name(), "Import Passkey");
        assert_eq!(WizardStep::PasskeyConfirm.name(), "Confirm Passkey");
        assert_eq!(WizardStep::MasterPassword.name(), "Master Password");
        assert_eq!(WizardStep::Complete.name(), "Complete");
    }

    #[test]
    fn test_wizard_state_new() {
        let state = WizardState::new();
        assert_eq!(state.step, WizardStep::Welcome);
        assert!(!state.can_proceed());
    }

    #[test]
    fn test_wizard_state_set_choice() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        assert!(state.can_proceed());
    }

    #[test]
    fn test_wizard_state_next_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);

        // Welcome -> Generate
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyGenerate);

        // Stay on Generate until words set
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyGenerate);

        // Add words, now can proceed
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyConfirm);
    }

    #[test]
    fn test_wizard_state_import_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::ImportExisting);

        state.next();
        assert_eq!(state.step, WizardStep::PasskeyImport);

        // Import -> Password (no confirmation needed)
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.next();
        assert_eq!(state.step, WizardStep::MasterPassword);
    }

    #[test]
    fn test_wizard_state_password_validation() {
        let mut state = WizardState::new();
        state.step = WizardStep::MasterPassword;

        // Can't proceed with short password
        state.set_master_password("short".to_string());
        assert!(!state.can_proceed());

        // Can proceed with 8+ char password
        state.set_master_password("longenough".to_string());
        assert!(state.can_proceed());
    }

    #[test]
    fn test_wizard_state_back_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.confirmed = true;

        state.step = WizardStep::MasterPassword;
        state.back();
        assert_eq!(state.step, WizardStep::PasskeyConfirm);
    }

    #[test]
    fn test_wizard_state_complete() {
        let mut state = WizardState::new();
        state.passkey_choice = Some(WelcomeChoice::GenerateNew);
        state.passkey_words = Some(vec!["word".to_string(); 24]);
        state.master_password = Some("securepassword".to_string());
        state.step = WizardStep::Complete;

        assert!(state.is_complete());
    }

    #[test]
    fn test_wizard_state_with_keystore_path() {
        let path = PathBuf::from("/test/path");
        let state = WizardState::new().with_keystore_path(path.clone());

        assert_eq!(state.require_keystore_path(), Some(&path));
    }
}
