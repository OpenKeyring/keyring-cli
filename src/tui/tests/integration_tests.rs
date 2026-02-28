//! TUI Integration Tests
//!
//! These tests verify complete multi-screen flows and interactions
//! between TuiApp and the various wizard screens.

use crate::tui::screens::welcome::WelcomeChoice;
use crate::tui::screens::wizard::{WizardState, WizardStep};
use crate::tui::screens::PasskeyVerifyScreen;
use crate::tui::testing::{render_snapshot, SnapshotSequence};
use crate::tui::{Screen, TuiApp};

#[test]
fn test_wizard_generate_flow_integration() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("wizard_generate_flow");

    // Start wizard by setting up state
    app.wizard_state =
        Some(WizardState::new().with_keystore_path(std::path::PathBuf::from("/test/path")));
    app.navigate_to(Screen::Wizard);

    // Initial wizard render (Welcome screen)
    let initial = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("wizard_welcome", initial);

    // Make choice and move to generate
    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.next();
        // Set words through the state
        state.set_passkey_words(vec!["word".to_string(); 24]);
    }
    app.passkey_generate_screen
        .set_words(vec!["word".to_string(); 24]);

    let generate_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("wizard_generate", generate_screen);

    // Move to confirm
    if let Some(state) = &mut app.wizard_state {
        state.next();
    }
    app.passkey_verify_screen = Some(PasskeyVerifyScreen::new(vec!["word".to_string(); 24]));

    let confirm_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("wizard_confirm", confirm_screen);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_wizard_import_flow_integration() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("wizard_import_flow");

    // Start wizard
    app.wizard_state =
        Some(WizardState::new().with_keystore_path(std::path::PathBuf::from("/test/path")));
    app.navigate_to(Screen::Wizard);

    // Make import choice
    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::ImportExisting);
        state.next();
    }

    // Import screen
    let import_screen = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("wizard_import", import_screen);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_tuiapp_to_wizard_transition() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("app_to_wizard_transition");

    // Main screen render (already at Main by default)
    let main = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("main_screen", main);

    // Transition to wizard
    app.wizard_state = Some(WizardState::new());
    app.navigate_to(Screen::Wizard);

    let wizard = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("wizard_screen", wizard);

    // Back to main
    app.return_to_main();

    let back_to_main = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("back_to_main", back_to_main);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_wizard_back_navigation_flow() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("wizard_back_flow");

    // Setup wizard at MasterPassword step
    app.wizard_state = Some(WizardState::new());
    app.navigate_to(Screen::Wizard);

    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.confirmed = true;
        state.step = WizardStep::MasterPassword;
        state.set_master_password("password123".to_string());
    }
    app.passkey_verify_screen = Some(PasskeyVerifyScreen::new(vec!["word".to_string(); 24]));

    // At MasterPassword
    let at_password = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("at_master_password", at_password);

    // Go back to PasskeyVerify
    if let Some(state) = &mut app.wizard_state {
        state.back();
    }

    let at_confirm = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("back_to_confirm", at_confirm);

    // Go back to PasskeyGenerate
    if let Some(state) = &mut app.wizard_state {
        state.back();
    }

    let at_generate = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("back_to_generate", at_generate);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_complete_onboarding_flow_snapshots() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("complete_onboarding");

    // Start onboarding
    app.wizard_state = Some(WizardState::new());
    app.navigate_to(Screen::Wizard);

    // Step 1: Welcome
    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
    }
    let welcome = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("welcome", welcome);

    // Step 2: Generate
    if let Some(state) = &mut app.wizard_state {
        state.next();
        state.set_passkey_words(vec!["word".to_string(); 24]);
    }
    app.passkey_generate_screen
        .set_words(vec!["word".to_string(); 24]);
    let generate = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("generate", generate);

    // Step 3: Confirm
    if let Some(state) = &mut app.wizard_state {
        state.next();
    }
    app.passkey_verify_screen = Some(PasskeyVerifyScreen::new(vec!["word".to_string(); 24]));
    let confirm = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("confirm", confirm);

    // Step 4: Master Password
    if let Some(state) = &mut app.wizard_state {
        state.toggle_confirmed();
        state.next();
        state.set_master_password("secure_password_123".to_string());
    }
    let master_password = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("master_password", master_password);

    // Step 5: Complete
    if let Some(state) = &mut app.wizard_state {
        state.next();
    }
    let complete = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("complete", complete);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_wizard_validation_states() {
    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("wizard_validations");

    // Setup at MasterPassword step with short password
    app.wizard_state = Some(WizardState::new());
    app.navigate_to(Screen::Wizard);

    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.confirmed = true;
        state.step = WizardStep::MasterPassword;
        state.set_master_password("short".to_string());
    }

    let short_password = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("short_password", short_password);

    // Valid password
    if let Some(state) = &mut app.wizard_state {
        state.set_master_password("longenough".to_string());
    }

    let valid_password = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("valid_password", valid_password);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_screen_sizes_integration() {
    let mut app = TuiApp::new();

    // Setup wizard at confirm step
    app.wizard_state = Some(WizardState::new());
    app.navigate_to(Screen::Wizard);

    if let Some(state) = &mut app.wizard_state {
        state.set_passkey_choice(WelcomeChoice::GenerateNew);
        state.set_passkey_words(vec!["word".to_string(); 24]);
        state.next();
    }
    app.passkey_verify_screen = Some(PasskeyVerifyScreen::new(vec!["word".to_string(); 24]));

    let mut seq = SnapshotSequence::new("screen_sizes");

    // Small terminal
    let small = render_snapshot(60, 20, |frame| {
        app.render(frame);
    });
    seq.step("60x20", small);

    // Medium terminal
    let medium = render_snapshot(80, 24, |frame| {
        app.render(frame);
    });
    seq.step("80x24", medium);

    // Large terminal
    let large = render_snapshot(120, 30, |frame| {
        app.render(frame);
    });
    seq.step("120x30", large);

    insta::assert_snapshot!(seq.to_string());
}
