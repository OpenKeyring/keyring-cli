// tests/tui/action_handlers_test.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keyring_cli::tui::TuiApp;

#[test]
fn test_sync_now_action() {
    let mut app = TuiApp::new();

    // Handle F5 (SyncNow)
    let event = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
    app.handle_key_event(event);

    // Should have output about sync
    assert!(app
        .output_lines
        .iter()
        .any(|l| l.contains("Sync") || l.contains("同步")));
}

#[test]
fn test_open_settings_action() {
    let mut app = TuiApp::new();

    let event = KeyEvent::new(KeyCode::F(2), KeyModifiers::empty());
    app.handle_key_event(event);

    // Should mention settings
    assert!(app
        .output_lines
        .iter()
        .any(|l| l.contains("Settings") || l.contains("设置")));
}

#[test]
fn test_save_config_action() {
    let mut app = TuiApp::new();

    // Ctrl+S triggers SaveConfig
    let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.handle_key_event(event);

    // Should have some output (verify handler doesn't crash)
    assert!(!app.output_lines.is_empty());

    // Check for save-related messages
    let has_save_message = app
        .output_lines
        .iter()
        .any(|l| l.contains("✓") || l.contains("save") || l.contains("Save"));
    assert!(
        has_save_message,
        "Expected save-related message, got: {:?}",
        app.output_lines
    );
}
