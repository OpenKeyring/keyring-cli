//! Wizard State Snapshot Tests
//!
//! These tests use `insta` to snapshot the WizardState at various stages
//! of the onboarding flow to ensure state transitions work correctly.

use crate::tui::screens::welcome::WelcomeChoice;
use crate::tui::screens::wizard::{WizardState, WizardStep};

#[test]
fn test_wizard_initial_state() {
    let state = WizardState::new();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_with_generate_choice() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::GenerateNew);
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_with_import_choice() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::ImportExisting);
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_generate_flow_snapshots() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::GenerateNew);

    // Snapshot: Welcome -> PasskeyGenerate
    state.next();
    insta::assert_debug_snapshot!(state);

    // Set passkey words (simulating generation)
    state.set_passkey_words(vec![
        "abandon".to_string(),
        "ability".to_string(),
        "able".to_string(),
        "about".to_string(),
        "above".to_string(),
        "absent".to_string(),
        "absorb".to_string(),
        "abstract".to_string(),
        "absurd".to_string(),
        "abuse".to_string(),
        "access".to_string(),
        "accident".to_string(),
        "account".to_string(),
        "accuse".to_string(),
        "achieve".to_string(),
        "acid".to_string(),
        "acoustic".to_string(),
        "acquire".to_string(),
        "across".to_string(),
        "act".to_string(),
        "action".to_string(),
        "actor".to_string(),
        "actress".to_string(),
        "actual".to_string(),
    ]);

    // Snapshot: After setting words, still on PasskeyGenerate
    insta::assert_debug_snapshot!(state);

    // Snapshot: PasskeyGenerate -> PasskeyVerify
    state.next();
    insta::assert_debug_snapshot!(state);

    // Confirm the passkey
    state.toggle_confirmed();

    // Snapshot: After confirmation
    insta::assert_debug_snapshot!(state);

    // Snapshot: PasskeyVerify -> MasterPassword
    state.next();
    insta::assert_debug_snapshot!(state);

    // Set master password
    state.set_master_password("secure_password_123".to_string());

    // Snapshot: After setting password
    insta::assert_debug_snapshot!(state);

    // Snapshot: MasterPassword -> Complete
    state.next();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_import_flow_snapshots() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::ImportExisting);

    // Snapshot: Welcome -> PasskeyImport
    state.next();
    insta::assert_debug_snapshot!(state);

    // Set passkey words (simulating import validation)
    state.set_passkey_words(vec![
        "abandon".to_string(),
        "ability".to_string(),
        "able".to_string(),
        "about".to_string(),
        "above".to_string(),
        "absent".to_string(),
        "absorb".to_string(),
        "abstract".to_string(),
        "absurd".to_string(),
        "abuse".to_string(),
        "access".to_string(),
        "accident".to_string(),
        "account".to_string(),
        "accuse".to_string(),
        "achieve".to_string(),
        "acid".to_string(),
        "acoustic".to_string(),
        "acquire".to_string(),
        "across".to_string(),
        "act".to_string(),
        "action".to_string(),
        "actor".to_string(),
        "actress".to_string(),
        "actual".to_string(),
    ]);

    // Snapshot: After setting words
    insta::assert_debug_snapshot!(state);

    // Snapshot: PasskeyImport -> MasterPassword (skips confirmation)
    state.next();
    insta::assert_debug_snapshot!(state);

    // Set master password
    state.set_master_password("secure_password_123".to_string());

    // Snapshot: After setting password
    insta::assert_debug_snapshot!(state);

    // Snapshot: MasterPassword -> Complete
    state.next();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_back_navigation_snapshots() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::GenerateNew);
    state.set_passkey_words(vec!["word".to_string(); 24]);
    state.confirmed = true;
    state.set_master_password("secure_password_123".to_string());
    state.step = WizardStep::MasterPassword;

    // Snapshot: Current state on MasterPassword
    insta::assert_debug_snapshot!(state);

    // Snapshot: MasterPassword -> PasskeyVerify (going back)
    state.back();
    insta::assert_debug_snapshot!(state);

    // Snapshot: PasskeyVerify -> PasskeyGenerate
    state.back();
    insta::assert_debug_snapshot!(state);

    // Snapshot: PasskeyGenerate -> Welcome
    state.back();
    insta::assert_debug_snapshot!(state);

    // Can't go back from Welcome
    state.back();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_validation_snapshots() {
    let mut state = WizardState::new();
    state.step = WizardStep::MasterPassword;

    // Snapshot: Short password - cannot proceed
    state.set_master_password("short".to_string());
    insta::assert_debug_snapshot!(state);

    // Snapshot: Valid 8+ character password - can proceed
    state.set_master_password("longenough".to_string());
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_error_handling_snapshots() {
    let mut state = WizardState::new();

    // Snapshot: With error message
    state.set_error("Failed to generate passkey".to_string());
    insta::assert_debug_snapshot!(state);

    // Snapshot: After clearing error
    state.clear_error();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_reset_snapshot() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::GenerateNew);
    state.set_passkey_words(vec!["word".to_string(); 24]);
    state.set_master_password("password".to_string());
    state.confirmed = true;
    state.step = WizardStep::MasterPassword;

    // Snapshot: Full state before reset
    insta::assert_debug_snapshot!(state);

    // Snapshot: After reset
    state.reset();
    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_complete_state_snapshot() {
    let mut state = WizardState::new();
    state.passkey_choice = Some(WelcomeChoice::GenerateNew);
    state.passkey_words = Some(vec!["word".to_string(); 24]);
    state.master_password = Some("secure_password_123".to_string());
    state.step = WizardStep::Complete;

    insta::assert_debug_snapshot!(state);
}

#[test]
fn test_wizard_state_sequence() {
    use crate::tui::testing::SnapshotSequence;

    let mut seq = SnapshotSequence::new("wizard_generate_flow");

    let mut state = WizardState::new();
    seq.step("initial", format!("{:?}", state));

    state.set_passkey_choice(WelcomeChoice::GenerateNew);
    seq.step("choice_set", format!("{:?}", state));

    state.next();
    seq.step("to_generate", format!("{:?}", state));

    state.set_passkey_words(vec!["word".to_string(); 24]);
    seq.step("words_set", format!("{:?}", state));

    state.next();
    seq.step("to_confirm", format!("{:?}", state));

    state.toggle_confirmed();
    seq.step("confirmed", format!("{:?}", state));

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_wizard_step_transitions() {
    let mut state = WizardState::new();
    state.set_passkey_choice(WelcomeChoice::GenerateNew);

    // Test each step transition using snapshot of step name
    assert_eq!(state.step, WizardStep::Welcome);
    insta::assert_snapshot!(state.step.name());

    state.next();
    assert_eq!(state.step, WizardStep::PasskeyGenerate);
    insta::assert_snapshot!(state.step.name());

    state.set_passkey_words(vec!["word".to_string(); 24]);
    state.next();
    assert_eq!(state.step, WizardStep::PasskeyVerify);
    insta::assert_snapshot!(state.step.name());

    state.toggle_confirmed();
    state.next();
    assert_eq!(state.step, WizardStep::MasterPassword);
    insta::assert_snapshot!(state.step.name());

    state.set_master_password("password123".to_string());
    state.next();
    assert_eq!(state.step, WizardStep::Complete);
    insta::assert_snapshot!(state.step.name());
}
