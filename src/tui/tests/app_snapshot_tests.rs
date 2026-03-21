//! TuiApp Snapshot Tests
//!
//! These tests use `insta` to snapshot TuiApp state for screen navigation.

#![cfg(feature = "test-env")]

use crate::tui::{Screen, TuiApp};
use serial_test::serial;

#[test]
#[serial]
fn test_tuiapp_navigation_snapshots() {
    let _env = crate::tui::testing::TestSnapshotEnv::new();
    let mut app = TuiApp::new();

    // Initial screen
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Settings
    app.navigate_to(Screen::Settings);
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Help
    app.navigate_to(Screen::Help);
    insta::assert_snapshot!(app.current_screen().name());

    // Navigate to Sync
    app.navigate_to(Screen::Sync);
    insta::assert_snapshot!(app.current_screen().name());

    // Return to main
    app.return_to_main();
    insta::assert_snapshot!(app.current_screen().name());
}

#[test]
#[serial]
fn test_tuiapp_screen_navigation_sequence() {
    let _env = crate::tui::testing::TestSnapshotEnv::new();
    use crate::tui::testing::SnapshotSequence;

    let mut app = TuiApp::new();
    let mut seq = SnapshotSequence::new("screen_navigation");

    seq.step("initial_screen", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Settings);
    seq.step("to_settings", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Help);
    seq.step("to_help", format!("{:?}", app.current_screen()));

    app.navigate_to(Screen::Sync);
    seq.step("to_sync", format!("{:?}", app.current_screen()));

    app.return_to_main();
    seq.step("back_to_main", format!("{:?}", app.current_screen()));

    insta::assert_snapshot!(seq.to_string());
}

#[test]
#[serial]
fn test_tuiapp_quit() {
    let _env = crate::tui::testing::TestSnapshotEnv::new();
    let mut app = TuiApp::new();

    assert!(app.is_running());
    app.quit();
    assert!(!app.is_running());
}
