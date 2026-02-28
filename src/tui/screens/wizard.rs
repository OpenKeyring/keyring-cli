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

// ============================================================================
// Configuration Types
// ============================================================================

/// Clipboard auto-clear timeout options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardTimeout {
    /// 10 seconds
    Seconds10,
    /// 30 seconds (recommended default)
    #[default]
    Seconds30,
    /// 60 seconds
    Seconds60,
}

impl ClipboardTimeout {
    /// Get timeout in seconds
    pub fn seconds(&self) -> u64 {
        match self {
            Self::Seconds10 => 10,
            Self::Seconds30 => 30,
            Self::Seconds60 => 60,
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Seconds10 => "10 seconds",
            Self::Seconds30 => "30 seconds (recommended)",
            Self::Seconds60 => "60 seconds",
        }
    }

    /// Get all variants for iteration
    pub const fn values() -> &'static [Self] {
        &[Self::Seconds10, Self::Seconds30, Self::Seconds60]
    }

    /// Move to next option (down navigation)
    pub fn next(&self) -> Self {
        let values = Self::values();
        let pos = values.iter().position(|&v| v == *self).unwrap_or(0);
        values[(pos + 1) % values.len()]
    }

    /// Move to previous option (up navigation)
    pub fn prev(&self) -> Self {
        let values = Self::values();
        let pos = values.iter().position(|&v| v == *self).unwrap_or(0);
        values[(pos + values.len() - 1) % values.len()]
    }
}

/// Trash retention period options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrashRetention {
    /// 7 days
    Days7,
    /// 30 days (default)
    #[default]
    Days30,
    /// 90 days
    Days90,
}

impl TrashRetention {
    /// Get retention in days
    pub fn days(&self) -> u32 {
        match self {
            Self::Days7 => 7,
            Self::Days30 => 30,
            Self::Days90 => 90,
        }
    }

    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Days7 => "7 days",
            Self::Days30 => "30 days (recommended)",
            Self::Days90 => "90 days",
        }
    }

    /// Get all variants for iteration
    pub const fn values() -> &'static [Self] {
        &[Self::Days7, Self::Days30, Self::Days90]
    }

    /// Move to next option (down navigation)
    pub fn next(&self) -> Self {
        let values = Self::values();
        let pos = values.iter().position(|&v| v == *self).unwrap_or(0);
        values[(pos + 1) % values.len()]
    }

    /// Move to previous option (up navigation)
    pub fn prev(&self) -> Self {
        let values = Self::values();
        let pos = values.iter().position(|&v| v == *self).unwrap_or(0);
        values[(pos + values.len() - 1) % values.len()]
    }
}

/// Default password generation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PasswordType {
    /// Random characters
    #[default]
    Random,
    /// Memorable word-based password
    Memorable,
    /// PIN code (numbers only)
    Pin,
}

impl PasswordType {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Random => "Random Password",
            Self::Memorable => "Memorable (Word-based)",
            Self::Pin => "PIN Code",
        }
    }
}

/// Password generation policy configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordPolicyConfig {
    /// Default password type
    pub default_type: PasswordType,
    /// Default password length (8-64)
    pub default_length: u8,
    /// Minimum number of digits
    pub min_digits: u8,
    /// Minimum number of special characters
    pub min_special: u8,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            default_type: PasswordType::default(),
            default_length: 16,
            min_digits: 2,
            min_special: 1,
        }
    }
}

// ============================================================================
// Wizard State
// ============================================================================

/// Complete state for the onboarding wizard
#[derive(Debug, Clone)]
pub struct WizardState {
    // === Current Step ===
    /// Current step in the wizard
    pub step: WizardStep,

    // === Passkey Data ===
    /// User's choice for Passkey setup
    pub passkey_choice: Option<WelcomeChoice>,
    /// The generated or imported Passkey words
    pub passkey_words: Option<Vec<String>>,

    // === Master Password ===
    /// Master password input (first entry)
    pub master_password: Option<String>,
    /// Master password confirmation (second entry)
    pub master_password_confirm: Option<String>,

    // === Passkey Verification ===
    /// 3 random positions to verify (1-indexed)
    pub verify_positions: Option<[usize; 3]>,
    /// User's answers for verification
    pub verify_answers: Option<[String; 3]>,

    // === Configuration ===
    /// Password generation policy
    pub password_policy: PasswordPolicyConfig,
    /// Clipboard auto-clear timeout
    pub clipboard_timeout: ClipboardTimeout,
    /// Trash retention period
    pub trash_retention: TrashRetention,
    /// Whether to skip password import
    pub skip_import: bool,

    // === Legacy/Other Fields ===
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
            master_password_confirm: None,
            verify_positions: None,
            verify_answers: None,
            password_policy: PasswordPolicyConfig::default(),
            clipboard_timeout: ClipboardTimeout::default(),
            trash_retention: TrashRetention::default(),
            skip_import: true,
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
            // === Entry ===
            WizardStep::Welcome => {
                // New setup starts with master password, import starts with passkey import
                if let Some(WelcomeChoice::GenerateNew) = self.passkey_choice {
                    WizardStep::MasterPassword
                } else {
                    WizardStep::PasskeyImport
                }
            }

            // === New Setup Flow ===
            WizardStep::MasterPassword => {
                if self.master_password.is_some() {
                    WizardStep::MasterPasswordConfirm
                } else {
                    WizardStep::MasterPassword
                }
            }
            WizardStep::MasterPasswordConfirm => {
                if self.passwords_match() {
                    WizardStep::SecurityNotice
                } else {
                    WizardStep::MasterPasswordConfirm
                }
            }
            WizardStep::SecurityNotice => WizardStep::PasskeyGenerate,
            WizardStep::PasskeyGenerate => {
                if self.passkey_words.is_some() {
                    WizardStep::PasskeyVerify
                } else {
                    WizardStep::PasskeyGenerate
                }
            }
            WizardStep::PasskeyVerify => {
                if self.verify_passkey() {
                    WizardStep::PasswordPolicy
                } else {
                    WizardStep::PasskeyVerify
                }
            }

            // === Import Flow ===
            WizardStep::PasskeyImport => {
                if self.passkey_words.is_some() {
                    WizardStep::MasterPasswordImport
                } else {
                    WizardStep::PasskeyImport
                }
            }
            WizardStep::MasterPasswordImport => {
                if self.master_password.is_some() {
                    WizardStep::MasterPasswordImportConfirm
                } else {
                    WizardStep::MasterPasswordImport
                }
            }
            WizardStep::MasterPasswordImportConfirm => {
                if self.passwords_match() {
                    WizardStep::PasswordHint
                } else {
                    WizardStep::MasterPasswordImportConfirm
                }
            }
            WizardStep::PasswordHint => WizardStep::PasswordPolicy,

            // === Common Configuration ===
            WizardStep::PasswordPolicy => WizardStep::ClipboardTimeout,
            WizardStep::ClipboardTimeout => WizardStep::TrashRetention,
            WizardStep::TrashRetention => WizardStep::ImportPasswords,
            WizardStep::ImportPasswords => WizardStep::Complete,

            // === Completion ===
            WizardStep::Complete => WizardStep::Complete,
        };
    }

    /// Go back to the previous step
    pub fn back(&mut self) {
        self.step = match self.step {
            // === Entry ===
            WizardStep::Welcome => WizardStep::Welcome,

            // === New Setup Flow ===
            WizardStep::MasterPassword => WizardStep::Welcome,
            WizardStep::MasterPasswordConfirm => WizardStep::MasterPassword,
            WizardStep::SecurityNotice => WizardStep::MasterPasswordConfirm,
            WizardStep::PasskeyGenerate => WizardStep::SecurityNotice,
            WizardStep::PasskeyVerify => WizardStep::PasskeyGenerate,

            // === Import Flow ===
            WizardStep::PasskeyImport => WizardStep::Welcome,
            WizardStep::MasterPasswordImport => WizardStep::PasskeyImport,
            WizardStep::MasterPasswordImportConfirm => WizardStep::MasterPasswordImport,
            WizardStep::PasswordHint => WizardStep::MasterPasswordImportConfirm,

            // === Common Configuration ===
            WizardStep::PasswordPolicy => {
                if let Some(WelcomeChoice::GenerateNew) = self.passkey_choice {
                    WizardStep::PasskeyVerify
                } else {
                    WizardStep::PasswordHint
                }
            }
            WizardStep::ClipboardTimeout => WizardStep::PasswordPolicy,
            WizardStep::TrashRetention => WizardStep::ClipboardTimeout,
            WizardStep::ImportPasswords => WizardStep::TrashRetention,

            // === Completion ===
            WizardStep::Complete => WizardStep::ImportPasswords,
        };
    }

    /// Check if we can proceed to the next step
    pub fn can_proceed(&self) -> bool {
        match self.step {
            // === Entry ===
            WizardStep::Welcome => self.passkey_choice.is_some(),

            // === New Setup Flow ===
            WizardStep::MasterPassword => {
                self.master_password.as_ref().map(|p| p.len() >= 8).unwrap_or(false)
            }
            WizardStep::MasterPasswordConfirm => self.passwords_match(),
            WizardStep::SecurityNotice => true,
            WizardStep::PasskeyGenerate => self.passkey_words.is_some(),
            WizardStep::PasskeyVerify => self.verify_passkey(),

            // === Import Flow ===
            WizardStep::PasskeyImport => self.passkey_words.is_some(),
            WizardStep::MasterPasswordImport => {
                self.master_password.as_ref().map(|p| p.len() >= 8).unwrap_or(false)
            }
            WizardStep::MasterPasswordImportConfirm => self.passwords_match(),
            WizardStep::PasswordHint => true,

            // === Common Configuration ===
            WizardStep::PasswordPolicy => true,
            WizardStep::ClipboardTimeout => true,
            WizardStep::TrashRetention => true,
            WizardStep::ImportPasswords => true,

            // === Completion ===
            WizardStep::Complete => true,
        }
    }

    /// Check if we can go back from current step
    pub fn can_go_back(&self) -> bool {
        !matches!(self.step, WizardStep::Welcome)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Check if master password and confirmation match
    pub fn passwords_match(&self) -> bool {
        match (&self.master_password, &self.master_password_confirm) {
            (Some(p1), Some(p2)) => p1 == p2 && p1.len() >= 8,
            _ => false,
        }
    }

    /// Verify passkey answers against expected words
    pub fn verify_passkey(&self) -> bool {
        match (&self.verify_positions, &self.verify_answers, &self.passkey_words) {
            (Some(positions), Some(answers), Some(words)) => {
                for (i, &pos) in positions.iter().enumerate() {
                    if i < answers.len() && pos > 0 && pos <= words.len() {
                        if answers[i].to_lowercase().trim() != words[pos - 1].to_lowercase().trim() {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Generate random verification positions
    pub fn generate_verify_positions(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();
        let mut positions = [0usize; 3];

        for i in 0..3 {
            loop {
                let pos = rng.random_range(1..=24);
                if !positions.contains(&pos) {
                    positions[i] = pos;
                    break;
                }
            }
        }

        self.verify_positions = Some(positions);
        self.verify_answers = Some([String::new(), String::new(), String::new()]);
    }

    /// Set answer for a verification position
    pub fn set_verify_answer(&mut self, index: usize, answer: String) {
        if let Some(answers) = &mut self.verify_answers {
            if index < 3 {
                answers[index] = answer;
            }
        }
    }

    // ========================================================================
    // Setters
    // ========================================================================

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

    /// Set the master password confirmation
    pub fn set_master_password_confirm(&mut self, password: String) {
        self.master_password_confirm = Some(password);
    }

    /// Set the confirmed state
    pub fn set_confirmed(&mut self, confirmed: bool) {
        self.confirmed = confirmed;
    }

    /// Toggle the confirmed state
    pub fn toggle_confirmed(&mut self) {
        self.confirmed = !self.confirmed;
    }

    /// Set clipboard timeout
    pub fn set_clipboard_timeout(&mut self, timeout: ClipboardTimeout) {
        self.clipboard_timeout = timeout;
    }

    /// Set trash retention
    pub fn set_trash_retention(&mut self, retention: TrashRetention) {
        self.trash_retention = retention;
    }

    /// Set password policy
    pub fn set_password_policy(&mut self, policy: PasswordPolicyConfig) {
        self.password_policy = policy;
    }

    /// Set skip import flag
    pub fn set_skip_import(&mut self, skip: bool) {
        self.skip_import = skip;
    }

    /// Set an error message
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// Clear any error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Check if wizard is complete
    pub fn is_complete(&self) -> bool {
        self.step == WizardStep::Complete
            && self.passkey_choice.is_some()
            && self.passkey_words.is_some()
            && self.passwords_match()
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

    /// Get clipboard timeout configuration
    pub fn clipboard_timeout(&self) -> ClipboardTimeout {
        self.clipboard_timeout
    }

    /// Get trash retention configuration
    pub fn trash_retention(&self) -> TrashRetention {
        self.trash_retention
    }

    /// Get password policy configuration
    pub fn password_policy(&self) -> PasswordPolicyConfig {
        self.password_policy
    }

    /// Check if import is skipped
    pub fn skip_import(&self) -> bool {
        self.skip_import
    }

    // ========================================================================
    // Reset
    // ========================================================================

    /// Reset the wizard state (useful for retry)
    pub fn reset(&mut self) {
        self.step = WizardStep::Welcome;
        self.passkey_choice = None;
        self.passkey_words = None;
        self.master_password = None;
        self.master_password_confirm = None;
        self.verify_positions = None;
        self.verify_answers = None;
        self.password_policy = PasswordPolicyConfig::default();
        self.clipboard_timeout = ClipboardTimeout::default();
        self.trash_retention = TrashRetention::default();
        self.skip_import = true;
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
    fn test_wizard_step_names() {
        assert_eq!(WizardStep::Welcome.name(), "Welcome");
        assert_eq!(WizardStep::MasterPassword.name(), "Master Password");
        assert_eq!(WizardStep::MasterPasswordConfirm.name(), "Confirm Password");
        assert_eq!(WizardStep::SecurityNotice.name(), "Security Notice");
        assert_eq!(WizardStep::PasskeyGenerate.name(), "Generate PassKey");
        assert_eq!(WizardStep::PasskeyVerify.name(), "Verify PassKey");
        assert_eq!(WizardStep::PasskeyImport.name(), "Import PassKey");
        assert_eq!(WizardStep::Complete.name(), "Complete");
    }

    #[test]
    fn test_config_defaults() {
        assert_eq!(ClipboardTimeout::default(), ClipboardTimeout::Seconds30);
        assert_eq!(TrashRetention::default(), TrashRetention::Days30);
        assert_eq!(PasswordType::default(), PasswordType::Random);

        let policy = PasswordPolicyConfig::default();
        assert_eq!(policy.default_length, 16);
        assert_eq!(policy.min_digits, 2);
        assert_eq!(policy.min_special, 1);
    }

    #[test]
    fn test_clipboard_timeout_seconds() {
        assert_eq!(ClipboardTimeout::Seconds10.seconds(), 10);
        assert_eq!(ClipboardTimeout::Seconds30.seconds(), 30);
        assert_eq!(ClipboardTimeout::Seconds60.seconds(), 60);
    }

    #[test]
    fn test_trash_retention_days() {
        assert_eq!(TrashRetention::Days7.days(), 7);
        assert_eq!(TrashRetention::Days30.days(), 30);
        assert_eq!(TrashRetention::Days90.days(), 90);
    }

    #[test]
    fn test_wizard_state_new() {
        let state = WizardState::new();
        assert_eq!(state.step, WizardStep::Welcome);
        assert!(!state.can_proceed());
        assert!(state.master_password.is_none());
        assert!(state.master_password_confirm.is_none());
    }

    #[test]
    fn test_wizard_state_set_choice() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        assert!(state.can_proceed());
    }

    #[test]
    fn test_passwords_match() {
        let mut state = WizardState::new();
        assert!(!state.passwords_match());

        state.set_master_password("password123".to_string());
        assert!(!state.passwords_match());

        state.set_master_password_confirm("password123".to_string());
        assert!(state.passwords_match());

        state.set_master_password_confirm("different".to_string());
        assert!(!state.passwords_match());
    }

    #[test]
    fn test_new_setup_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);

        // Welcome -> MasterPassword
        state.next();
        assert_eq!(state.step, WizardStep::MasterPassword);

        // Need password to proceed
        state.set_master_password("longenough".to_string());
        state.next();
        assert_eq!(state.step, WizardStep::MasterPasswordConfirm);

        // Need matching confirmation
        state.set_master_password_confirm("longenough".to_string());
        state.next();
        assert_eq!(state.step, WizardStep::SecurityNotice);

        // SecurityNotice -> PasskeyGenerate
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyGenerate);

        // Need passkey words
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyVerify);
    }

    #[test]
    fn test_import_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::ImportExisting);

        // Welcome -> PasskeyImport
        state.next();
        assert_eq!(state.step, WizardStep::PasskeyImport);

        // Need words to proceed
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.next();
        assert_eq!(state.step, WizardStep::MasterPasswordImport);

        // Set password
        state.set_master_password("longenough".to_string());
        state.next();
        assert_eq!(state.step, WizardStep::MasterPasswordImportConfirm);

        // Confirm password
        state.set_master_password_confirm("longenough".to_string());
        state.next();
        assert_eq!(state.step, WizardStep::PasswordHint);

        // Continue through config steps
        state.next();
        assert_eq!(state.step, WizardStep::PasswordPolicy);
        state.next();
        assert_eq!(state.step, WizardStep::ClipboardTimeout);
        state.next();
        assert_eq!(state.step, WizardStep::TrashRetention);
        state.next();
        assert_eq!(state.step, WizardStep::ImportPasswords);
        state.next();
        assert_eq!(state.step, WizardStep::Complete);
    }

    #[test]
    fn test_password_validation() {
        let mut state = WizardState::new();
        state.step = WizardStep::MasterPassword;

        // Short password fails
        state.set_master_password("short".to_string());
        assert!(!state.can_proceed());

        // Long enough password passes
        state.set_master_password("longenough".to_string());
        assert!(state.can_proceed());
    }

    #[test]
    fn test_back_flow() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.step = WizardStep::PasskeyVerify;

        state.back();
        assert_eq!(state.step, WizardStep::PasskeyGenerate);

        state.back();
        assert_eq!(state.step, WizardStep::SecurityNotice);

        state.back();
        assert_eq!(state.step, WizardStep::MasterPasswordConfirm);
    }

    #[test]
    fn test_reset() {
        let mut state = WizardState::new();
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.set_master_password("password".to_string());
        state.set_master_password_confirm("password".to_string());
        state.step = WizardStep::PasskeyVerify;

        state.reset();
        assert_eq!(state.step, WizardStep::Welcome);
        assert!(state.passkey_choice.is_none());
        assert!(state.master_password.is_none());
        assert!(state.master_password_confirm.is_none());
    }

    #[test]
    fn test_wizard_state_with_keystore_path() {
        let path = PathBuf::from("/test/path");
        let state = WizardState::new().with_keystore_path(path.clone());
        assert_eq!(state.require_keystore_path(), Some(&path));
    }
}
