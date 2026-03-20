//! Tests for Wizard State
//!
//! Unit tests for the onboarding wizard state machine.

use super::*;
use crate::tui::screens::welcome::WelcomeChoice;
use std::path::PathBuf;

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
