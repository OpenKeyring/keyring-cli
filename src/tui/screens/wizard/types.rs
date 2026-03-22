//! Wizard Types
//!
//! Type definitions for the onboarding wizard, including step enum
//! and configuration types.

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
