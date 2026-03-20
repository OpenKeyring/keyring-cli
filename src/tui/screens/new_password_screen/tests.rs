//! Unit tests for NewPasswordScreen

use super::{FormField, NewPasswordScreen};

#[test]
fn test_new_password_screen_creation() {
    let screen = NewPasswordScreen::new();
    assert!(!screen.password.is_empty());
}

#[test]
fn test_validation_empty_name() {
    let screen = NewPasswordScreen::new();
    let result = screen.validate();
    assert!(result.is_err());
}

#[test]
fn test_validation_with_name() {
    let mut screen = NewPasswordScreen::new();
    screen.name = "Test Password".to_string();
    let result = screen.validate();
    assert!(result.is_ok());
}

#[test]
fn test_password_generation() {
    let mut screen = NewPasswordScreen::new();
    screen.name = "Test".to_string();
    screen.password_length = 20;
    screen.generate_password();
    assert_eq!(screen.password.len(), 20);
}

#[test]
fn test_password_toggle_visibility() {
    let mut screen = NewPasswordScreen::new();
    assert!(!screen.password_visible);
    screen.password_visible = true;
    assert!(screen.password_visible);
}

#[test]
fn test_get_password_record() {
    let mut screen = NewPasswordScreen::new();
    screen.name = "Test Password".to_string();
    let record = screen.get_password_record();
    assert!(record.is_some());
    let rec = record.unwrap();
    assert_eq!(rec.name, "Test Password");
}

#[test]
fn test_form_field_index() {
    assert_eq!(FormField::Name.index(), 0);
    assert_eq!(FormField::Username.index(), 1);
    assert_eq!(FormField::PasswordType.index(), 2);
    assert_eq!(FormField::PasswordLength.index(), 3);
    assert_eq!(FormField::Password.index(), 4);
    assert_eq!(FormField::Url.index(), 5);
    assert_eq!(FormField::Notes.index(), 6);
    assert_eq!(FormField::Tags.index(), 7);
    assert_eq!(FormField::Group.index(), 8);
}

#[test]
fn test_form_field_from_index() {
    assert_eq!(FormField::from_index(0), Some(FormField::Name));
    assert_eq!(FormField::from_index(4), Some(FormField::Password));
    assert_eq!(FormField::from_index(8), Some(FormField::Group));
    assert_eq!(FormField::from_index(9), None);
}

#[test]
fn test_form_field_required() {
    assert!(FormField::Name.is_required());
    assert!(!FormField::Username.is_required());
    assert!(!FormField::Password.is_required());
}

#[test]
fn test_form_field_label() {
    assert_eq!(FormField::Name.label(), "Name");
    assert_eq!(FormField::PasswordType.label(), "Password Type");
    assert_eq!(FormField::PasswordLength.label(), "Length");
}
