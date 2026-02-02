//! Screen Snapshot Tests
//!
//! These tests provide snapshot coverage for individual screen components
//! including PasskeyGenerate, PasskeyImport, and PasskeyConfirm screens.

use crate::tui::screens::{
    PasskeyConfirmScreen, PasskeyGenerateScreen, PasskeyImportScreen,
};
use crate::tui::testing::{render_snapshot, SnapshotSequence};

#[test]
fn test_passkey_generate_initial_state() {
    let screen = PasskeyGenerateScreen::new();
    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_generate_with_12_words() {
    let screen = PasskeyGenerateScreen::with_word_count(12);
    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_generate_render_initial() {
    let screen = PasskeyGenerateScreen::new();
    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_generate_render_with_words() {
    let mut screen = PasskeyGenerateScreen::new();
    screen.set_words(vec![
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

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_generate_render_confirmed() {
    let mut screen = PasskeyGenerateScreen::new();
    screen.set_words(vec!["word".to_string(); 24]);
    screen.set_confirmed(true);

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_generate_confirmation_sequence() {
    let mut screen = PasskeyGenerateScreen::new();
    let mut seq = SnapshotSequence::new("generate_confirmation_flow");

    // Initial state
    let initial = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("initial", initial);

    // After generating words
    screen.set_words(vec!["word".to_string(); 24]);
    let with_words = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("with_words", with_words);

    // After confirmation
    screen.toggle_confirm();
    let confirmed = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("confirmed", confirmed);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_passkey_import_initial_state() {
    let screen = PasskeyImportScreen::new();
    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_import_with_input() {
    let mut screen = PasskeyImportScreen::new();
    screen.handle_char('a');
    screen.handle_char('b');
    screen.handle_char('c');

    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_import_render_initial() {
    let screen = PasskeyImportScreen::new();
    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_import_render_with_input() {
    let mut screen = PasskeyImportScreen::new();
    for c in "abandon ability able about".chars() {
        screen.handle_char(c);
    }

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_import_render_validated() {
    let mut screen = PasskeyImportScreen::new();
    // Set valid 12-word BIP39 mnemonic (using known valid words)
    let valid_words = "abandon ability able about above absent absorb abstract absurd abuse access accident account accuse act";
    for c in valid_words.chars() {
        screen.handle_char(c);
    }
    let _ = screen.validate();

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_import_render_error() {
    let mut screen = PasskeyImportScreen::new();
    // Type insufficient words
    for c in "one two three".chars() {
        screen.handle_char(c);
    }
    let _ = screen.validate();

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_import_input_sequence() {
    let mut screen = PasskeyImportScreen::new();
    let mut seq = SnapshotSequence::new("import_input_sequence");

    // Type some words
    for c in "abandon ability able about".chars() {
        screen.handle_char(c);
    }

    let with_input = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("with_partial_input", with_input);

    // Backspace
    screen.handle_backspace();
    let after_backspace = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("after_backspace", after_backspace);

    insta::assert_snapshot!(seq.to_string());
}

#[test]
fn test_passkey_confirm_initial_state() {
    let words = vec!["word".to_string(); 24];
    let screen = PasskeyConfirmScreen::new(words);
    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_confirm_confirmed_state() {
    let words = vec!["word".to_string(); 24];
    let mut screen = PasskeyConfirmScreen::new(words);
    screen.toggle();

    insta::assert_debug_snapshot!(screen);
}

#[test]
fn test_passkey_confirm_render_initial() {
    let words = vec![
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
    ];
    let screen = PasskeyConfirmScreen::new(words);

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_confirm_render_confirmed() {
    let words = vec![
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
    ];
    let mut screen = PasskeyConfirmScreen::new(words);
    screen.toggle();

    let output = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });

    insta::assert_snapshot!(output);
}

#[test]
fn test_passkey_confirm_toggle_sequence() {
    let words = vec!["word".to_string(); 24];
    let mut screen = PasskeyConfirmScreen::new(words);
    let mut seq = SnapshotSequence::new("confirm_toggle_flow");

    // Initial state
    let initial = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("initial", initial);

    // After toggle
    screen.toggle();
    let confirmed = render_snapshot(80, 24, |frame| {
        screen.render(frame, frame.area());
    });
    seq.step("confirmed", confirmed);

    insta::assert_snapshot!(seq.to_string());
}
