//! Unit tests for EditPasswordScreen

use super::{EditedPasswordFields, EditFormField, EditPasswordScreen};
use crate::tui::traits::{HandleResult, Interactive};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

#[test]
fn test_edit_password_screen_creation() {
    let id = Uuid::new_v4();
    let screen = EditPasswordScreen::new(
        id,
        "Test Password",
        Some("user@example.com"),
        "original_password",
        Some("https://example.com"),
        Some("Test notes"),
        &["tag1".to_string(), "tag2".to_string()],
        Some("Personal"),
    );

    assert_eq!(screen.password_name(), "Test Password");
    assert_eq!(screen.username, "user@example.com");
    assert!(screen.new_password.is_none()); // Should keep original
    assert_eq!(screen.url, "https://example.com");
}

#[test]
fn test_password_regeneration() {
    let id = Uuid::new_v4();
    let mut screen = EditPasswordScreen::new(
        id,
        "Test",
        None,
        "original",
        None,
        None,
        &[],
        None,
    );

    assert!(screen.new_password.is_none());
    screen.generate_password();
    assert!(screen.new_password.is_some());
    assert!(screen.is_password_changed());
}

#[test]
fn test_get_current_password() {
    let id = Uuid::new_v4();
    let mut screen = EditPasswordScreen::new(
        id,
        "Test",
        None,
        "original_password",
        None,
        None,
        &[],
        None,
    );

    // Initially returns original
    assert_eq!(screen.get_current_password(), "original_password");

    // After generation, returns new
    screen.generate_password();
    assert_ne!(screen.get_current_password(), "original_password");
}

#[test]
fn test_get_edited_fields() {
    let id = Uuid::new_v4();
    let screen = EditPasswordScreen::new(
        id,
        "Test",
        Some("user"),
        "pass",
        Some("https://example.com"),
        Some("notes"),
        &["tag1".to_string()],
        Some("Work"),
    );

    let fields = screen.get_edited_fields();
    assert_eq!(fields.id, id);
    assert_eq!(fields.username, Some("user".to_string()));
    assert_eq!(fields.url, Some("https://example.com".to_string()));
    assert_eq!(fields.notes, Some("notes".to_string()));
    assert_eq!(fields.group_id, Some("Work".to_string()));
}

#[test]
fn test_toggle_password_visibility() {
    let id = Uuid::new_v4();
    let mut screen = EditPasswordScreen::new(
        id,
        "Test",
        None,
        "password123",
        None,
        None,
        &[],
        None,
    );

    assert!(!screen.password_visible);

    // Simulate Space key on password field
    screen.focused_field = EditFormField::Password.index();
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty());
    let result = screen.handle_key(key);
    assert!(matches!(result, HandleResult::NeedsRender));
    assert!(screen.password_visible);
}

#[test]
fn test_edit_form_field_index() {
    assert_eq!(EditFormField::Username.index(), 0);
    assert_eq!(EditFormField::PasswordType.index(), 1);
    assert_eq!(EditFormField::PasswordLength.index(), 2);
    assert_eq!(EditFormField::Password.index(), 3);
    assert_eq!(EditFormField::Url.index(), 4);
    assert_eq!(EditFormField::Notes.index(), 5);
    assert_eq!(EditFormField::Tags.index(), 6);
    assert_eq!(EditFormField::Group.index(), 7);
}

#[test]
fn test_edit_form_field_from_index() {
    assert_eq!(EditFormField::from_index(0), Some(EditFormField::Username));
    assert_eq!(EditFormField::from_index(3), Some(EditFormField::Password));
    assert_eq!(EditFormField::from_index(7), Some(EditFormField::Group));
    assert_eq!(EditFormField::from_index(8), None);
}

#[test]
fn test_edit_form_field_label() {
    assert_eq!(EditFormField::Username.label(), "Username");
    assert_eq!(EditFormField::PasswordType.label(), "Password Type");
    assert_eq!(EditFormField::PasswordLength.label(), "Length");
}

#[test]
fn test_empty_screen() {
    let screen = EditPasswordScreen::empty();
    assert_eq!(screen.password_name(), "");
    assert!(screen.username.is_empty());
    assert!(screen.new_password.is_none());
}

#[test]
fn test_navigation_keys() {
    let mut screen = EditPasswordScreen::empty();

    // Tab navigation
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::empty());
    screen.handle_key(tab_key);
    assert_eq!(screen.focused_field, 1);

    // Down navigation
    let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
    screen.handle_key(down_key);
    assert_eq!(screen.focused_field, 2);

    // Up navigation
    let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
    screen.handle_key(up_key);
    assert_eq!(screen.focused_field, 1);
}

#[test]
fn test_text_input() {
    let mut screen = EditPasswordScreen::empty();
    screen.focused_field = EditFormField::Username.index();

    let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
    screen.handle_key(char_key);
    assert_eq!(screen.username, "a");

    let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty());
    screen.handle_key(backspace_key);
    assert!(screen.username.is_empty());
}
