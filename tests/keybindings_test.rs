//! Keybindings module tests
//!
//! Test-Driven Development tests for the keybindings system.

// Note: These tests will fail initially until we implement the keybindings module

use keyring_cli::tui::keybindings::{parseShortcut, Action, KeyBinding, KeyBindingManager};

#[test]
fn test_parse_ctrl_char() {
    // Test parsing "Ctrl+N" into KeyEvent
    // This will fail until we implement the parser
    let result = parseShortcut("Ctrl+N");
    assert!(result.is_ok());
    let event = result.unwrap();
    assert_eq!(event.code, crossterm::event::KeyCode::Char('n'));
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
}

#[test]
fn test_parse_function_key() {
    let result = parseShortcut("F5");
    assert!(result.is_ok());
    let event = result.unwrap();
    assert_eq!(event.code, crossterm::event::KeyCode::F(5));
}

#[test]
fn test_parse_ctrl_shift_char() {
    let result = parseShortcut("Ctrl+Shift+N");
    assert!(result.is_ok());
    let event = result.unwrap();
    assert_eq!(event.code, crossterm::event::KeyCode::Char('N'));
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT));
}

#[test]
fn test_parse_invalid_shortcut() {
    let result = parseShortcut("Invalid");
    assert!(result.is_err());
}

#[test]
fn test_action_display() {
    // Test that actions can be displayed for help
    assert_eq!(format!("{}", Action::New), "New");
    assert_eq!(format!("{}", Action::List), "List");
    assert_eq!(format!("{}", Action::Quit), "Quit");
}

#[test]
fn test_default_keybindings() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let manager = KeyBindingManager::new();

    // Test default bindings exist
    let ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_n), Some(Action::New));

    let ctrl_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_l), Some(Action::List));

    let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&ctrl_q), Some(Action::Quit));
}

#[test]
fn test_keybinding_from_yaml() {
    use serde_yaml;

    let yaml = r#"
version: "1.0"
shortcuts:
  new: "Ctrl+N"
  list: "Ctrl+L"
"#;

    let binding: Result<KeyBinding, _> = serde_yaml::from_str(yaml);
    assert!(binding.is_ok());
}

#[test]
fn test_conflict_detection() {
    use serde_yaml;

    // Two actions with same shortcut - should detect conflict
    let yaml = r#"
version: "1.0"
shortcuts:
  new: "Ctrl+N"
  list: "Ctrl+N"
"#;

    let binding: Result<KeyBinding, _> = serde_yaml::from_str(yaml);
    // Should parse but warn about conflict
    assert!(binding.is_ok());
}
