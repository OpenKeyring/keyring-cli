//! TUI Integration Tests
//!
//! The old CLI-style event routing (handle_key_event, output_lines) has been
//! removed as part of the TUI MVP refactoring. Screen navigation is now handled
//! through the unified event loop in terminal.rs.
//! These tests will be rewritten to test the new per-screen routing.

use keyring_cli::tui::{Screen, TuiApp};

#[test]
fn test_navigate_to_screen() {
    let mut app = TuiApp::new();

    app.navigate_to(Screen::Settings);
    assert_eq!(app.current_screen(), Screen::Settings);

    app.navigate_to(Screen::Help);
    assert_eq!(app.current_screen(), Screen::Help);

    app.navigate_to(Screen::Sync);
    assert_eq!(app.current_screen(), Screen::Sync);

    app.return_to_main();
    assert_eq!(app.current_screen(), Screen::Main);
}

#[test]
fn test_quit_stops_running() {
    let mut app = TuiApp::new();
    assert!(app.is_running());

    app.quit();
    assert!(!app.is_running());
}
