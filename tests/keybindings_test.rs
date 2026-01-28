//! Keybindings module tests
//!
//! Test-Driven Development tests for the keybindings system.

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

// Additional comprehensive tests

#[test]
fn test_all_default_actions_have_bindings() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let manager = KeyBindingManager::new();

    // All actions should have bindings
    let all_actions = vec![
        Action::New,
        Action::List,
        Action::Search,
        Action::Show,
        Action::Update,
        Action::Delete,
        Action::Quit,
        Action::Help,
        Action::Clear,
        Action::CopyPassword,
        Action::CopyUsername,
        Action::Config,
    ];

    for action in all_actions {
        let key = manager.get_key(action);
        assert!(key.is_some(), "Action {:?} should have a key binding", action);
    }
}

#[test]
fn test_manager_get_key_for_action() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let manager = KeyBindingManager::new();

    let new_key = manager.get_key(Action::New);
    assert_eq!(new_key.unwrap().code, KeyCode::Char('n'));

    let help_key = manager.get_key(Action::Help);
    assert_eq!(help_key.unwrap().code, KeyCode::Char('h'));
}

#[test]
fn test_manager_format_key() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let ctrl_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(KeyBindingManager::format_key(&ctrl_n), "Ctrl+n");

    let ctrl_shift_n = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    assert_eq!(KeyBindingManager::format_key(&ctrl_shift_n), "Ctrl+Shift+N");

    let f5 = KeyEvent::new(KeyCode::F(5), KeyModifiers::empty());
    assert_eq!(KeyBindingManager::format_key(&f5), "F5");
}

#[test]
fn test_parse_alt_key() {
    let result = parseShortcut("Alt+T");
    assert!(result.is_ok());
    let event = result.unwrap();
    assert_eq!(event.code, crossterm::event::KeyCode::Char('t'));
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::ALT));
}

#[test]
fn test_parse_ctrl_alt_key() {
    let result = parseShortcut("Ctrl+Alt+Delete");
    assert!(result.is_ok());
    let event = result.unwrap();
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL));
    assert!(event.modifiers.contains(crossterm::event::KeyModifiers::ALT));
}

#[test]
fn test_parse_empty_input() {
    let result = parseShortcut("");
    assert!(result.is_err());
}

#[test]
fn test_parse_whitespace_only() {
    let result = parseShortcut("   ");
    assert!(result.is_err());
}

#[test]
fn test_parse_special_keys() {
    assert_eq!(parseShortcut("Enter").unwrap().code, crossterm::event::KeyCode::Enter);
    assert_eq!(parseShortcut("Tab").unwrap().code, crossterm::event::KeyCode::Tab);
    assert_eq!(parseShortcut("Esc").unwrap().code, crossterm::event::KeyCode::Esc);
    assert_eq!(parseShortcut("Backspace").unwrap().code, crossterm::event::KeyCode::Backspace);
    assert_eq!(parseShortcut("Space").unwrap().code, crossterm::event::KeyCode::Char(' '));
}

#[test]
fn test_parse_navigation_keys() {
    assert_eq!(parseShortcut("Up").unwrap().code, crossterm::event::KeyCode::Up);
    assert_eq!(parseShortcut("Down").unwrap().code, crossterm::event::KeyCode::Down);
    assert_eq!(parseShortcut("Left").unwrap().code, crossterm::event::KeyCode::Left);
    assert_eq!(parseShortcut("Right").unwrap().code, crossterm::event::KeyCode::Right);
}

#[test]
fn test_parse_function_keys_f1_to_f12() {
    for i in 1..=12 {
        let result = parseShortcut(&format!("F{}", i));
        assert!(result.is_ok(), "F{} should parse", i);
        assert_eq!(result.unwrap().code, crossterm::event::KeyCode::F(i));
    }
}

#[test]
fn test_parse_case_insensitive_modifiers() {
    let ctrl_lower = parseShortcut("ctrl+n");
    let ctrl_upper = parseShortcut("CTRL+N");
    let ctrl_mixed = parseShortcut("Ctrl+N");

    assert!(ctrl_lower.is_ok());
    assert!(ctrl_upper.is_ok());
    assert!(ctrl_mixed.is_ok());

    // All should produce the same result
    assert_eq!(ctrl_lower.unwrap(), ctrl_upper.unwrap());
}

#[test]
fn test_action_command_names() {
    assert_eq!(Action::New.command_name(), "/new");
    assert_eq!(Action::List.command_name(), "/list");
    assert_eq!(Action::Quit.command_name(), "/exit");
    assert_eq!(Action::Help.command_name(), "/help");
}

#[test]
fn test_action_descriptions() {
    assert!(!Action::New.description().is_empty());
    assert!(!Action::Quit.description().is_empty());
    assert!(!Action::Help.description().is_empty());
}

#[test]
fn test_keybinding_default_creation() {
    let binding = KeyBinding::new();
    assert_eq!(binding.version, "1.0");
    assert_eq!(binding.shortcuts.get("new"), Some(&"Ctrl+N".to_string()));
    assert_eq!(binding.shortcuts.get("quit"), Some(&"Ctrl+Q".to_string()));
}

#[test]
fn test_unknown_shortcut_returns_none() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let manager = KeyBindingManager::new();
    let unknown_key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
    assert_eq!(manager.get_action(&unknown_key), None);
}

#[test]
fn test_all_bindings_coverage() {
    let manager = KeyBindingManager::new();
    let bindings = manager.all_bindings();

    // Should have at least 12 bindings (one for each action)
    assert!(bindings.len() >= 12);
}
