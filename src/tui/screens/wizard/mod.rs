//! Wizard State Management
//!
//! Core state machine for the onboarding wizard, managing the flow between
//! different wizard steps and collecting user data.

mod state;
mod types;

// Re-export public types
pub use state::WizardState;
pub use types::{
    ClipboardTimeout, PasswordPolicyConfig, PasswordType, TrashRetention, WizardStep,
};

#[cfg(test)]
mod tests;
