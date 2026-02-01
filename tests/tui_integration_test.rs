//! Integration tests for TUI screen navigation and routing
//!
//! These tests verify that:
//! - F2 key navigates to Settings screen
//! - F5 key navigates to Sync screen
//! - '?' key navigates to Help screen
//! - Screen-specific handlers are called correctly
//! - Navigation between screens works properly

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keyring_cli::tui::{Screen, TuiApp};

#[test]
fn test_f2_navigates_to_settings_screen() {
    let mut app = TuiApp::new();

    // Press F2 to navigate to settings
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);

    // Verify we're on the settings screen
    assert_eq!(app.current_screen(), Screen::Settings);
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("Settings")));
}

#[test]
fn test_f5_navigates_to_sync_screen() {
    let mut app = TuiApp::new();

    // Press F5 to navigate to sync
    let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
    app.handle_key_event(f5);

    // Verify we're on the sync screen
    assert_eq!(app.current_screen(), Screen::Sync);
    // Sync output should be shown
    assert!(app.output_lines.iter().any(|l: &String| l.contains("Sync")));
}

#[test]
fn test_question_mark_navigates_to_help_screen() {
    let mut app = TuiApp::new();

    // Press '?' to navigate to help
    let question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    app.handle_key_event(question);

    // Verify we're on the help screen
    assert_eq!(app.current_screen(), Screen::Help);
    assert!(app.output_lines.iter().any(|l: &String| l.contains("Help")
        || l.contains("Keyboard Shortcuts")
        || l.contains("Commands")));
}

#[test]
fn test_escape_returns_to_main_screen() {
    let mut app = TuiApp::new();

    // Navigate to settings first
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);
    assert_eq!(app.current_screen(), Screen::Settings);

    // Press Escape to return to main
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    app.handle_key_event(esc);

    // Verify we're back to main screen
    assert_eq!(app.current_screen(), Screen::Main);
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("Returned to main")));
}

#[test]
fn test_screen_navigation_sequence() {
    let mut app = TuiApp::new();

    // Navigate: Main -> Settings -> Help -> Sync
    // Then Esc to return to Main
    let screens_visited = vec![
        (KeyCode::F(2), Screen::Settings),
        (KeyCode::Char('?'), Screen::Help),
        (KeyCode::F(5), Screen::Sync), // F5 navigates to Sync screen
        (KeyCode::Esc, Screen::Main),  // Esc returns to Main
    ];

    for (key, expected_screen) in screens_visited {
        app.handle_key_event(KeyEvent::new(key, KeyModifiers::empty()));
        assert_eq!(app.current_screen(), expected_screen);
    }
}

#[test]
fn test_ctrl_n_works_on_all_screens() {
    let mut app = TuiApp::new();

    // Test Ctrl+N (New) works on main screen
    let ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    app.handle_key_event(ctrl_n);
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("> /new")));

    // Navigate to settings and test Ctrl+N still works
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);

    app.handle_key_event(ctrl_n);
    // Should still trigger new command regardless of screen
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("> /new")));
}

#[test]
fn test_ctrl_q_quit_works_from_any_screen() {
    let mut app = TuiApp::new();

    // Navigate to settings
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);

    // Press Ctrl+Q to quit
    let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    app.handle_key_event(ctrl_q);

    // App should quit regardless of current screen
    assert!(!app.is_running());
}

#[test]
fn test_screen_state_persistence() {
    let mut app = TuiApp::new();

    // Navigate to settings
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);
    assert_eq!(app.current_screen(), Screen::Settings);

    // Navigate away
    let help = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    app.handle_key_event(help);
    assert_eq!(app.current_screen(), Screen::Help);

    // Return to settings
    app.handle_key_event(f2);
    assert_eq!(app.current_screen(), Screen::Settings);
}

#[test]
fn test_multiple_screen_transitions() {
    let mut app = TuiApp::new();

    // Test rapid screen transitions (don't press Esc on Main as it quits)
    let transitions = vec![
        (KeyCode::F(2), Screen::Settings),  // Settings
        (KeyCode::Esc, Screen::Main),       // Return to Main
        (KeyCode::F(5), Screen::Sync),      // Navigate to Sync
        (KeyCode::Esc, Screen::Main),       // Return to Main
        (KeyCode::Char('?'), Screen::Help), // Help
        (KeyCode::F(2), Screen::Settings),  // Settings (from Help)
        (KeyCode::Esc, Screen::Main),       // Main (from Settings)
        (KeyCode::F(5), Screen::Sync),      // Navigate to Sync
                                            // Don't press Esc on Main as it would quit
    ];

    for (key, expected_screen) in transitions {
        app.handle_key_event(KeyEvent::new(key, KeyModifiers::empty()));
        assert_eq!(app.current_screen(), expected_screen);
    }

    // Should complete without panicking
    assert!(app.is_running());
}

#[test]
fn test_screen_routing_delegates_to_correct_handler() {
    let mut app = TuiApp::new();

    // Test that screen-specific handlers are called
    // Settings screen (F2)
    let f2 = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(f2);
    assert_eq!(app.current_screen(), Screen::Settings);
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("Settings")));

    // Help screen (?)
    let question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    app.handle_key_event(question);
    assert_eq!(app.current_screen(), Screen::Help);
    assert!(app
        .output_lines
        .iter()
        .any(|l: &String| l.contains("Keyboard Shortcuts")));

    // Return to main first
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    app.handle_key_event(esc);
    assert_eq!(app.current_screen(), Screen::Main);

    // Sync screen (F5)
    let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
    app.handle_key_event(f5);
    assert_eq!(app.current_screen(), Screen::Sync);
    assert!(app.output_lines.iter().any(|l: &String| l.contains("Sync")));
}
