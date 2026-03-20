//! Tests for MasterPasswordScreen
//!
//! Unit tests for the master password screen component.

use super::*;
use ratatui::style::Color;

#[test]
fn test_master_password_new() {
    let screen = MasterPasswordScreen::new();
    assert!(screen.is_showing_first());
    assert_eq!(screen.password_input(), "");
    assert_eq!(screen.confirm_input(), "");
}

#[test]
fn test_master_password_handle_char() {
    let mut screen = MasterPasswordScreen::new();
    screen.handle_char('a');
    screen.handle_char('b');
    screen.handle_char('c');
    assert_eq!(screen.password_input(), "abc");
}

#[test]
fn test_master_password_handle_backspace() {
    let mut screen = MasterPasswordScreen::new();
    screen.handle_char('a');
    screen.handle_char('b');
    screen.handle_backspace();
    assert_eq!(screen.password_input(), "a");
}

#[test]
fn test_master_password_next() {
    let mut screen = MasterPasswordScreen::new();
    screen.handle_char('a');
    screen.next();
    assert!(!screen.is_showing_first());
}

#[test]
fn test_master_password_back() {
    let mut screen = MasterPasswordScreen::new();
    screen.handle_char('a');
    screen.next();
    screen.back();
    assert!(screen.is_showing_first());
}

#[test]
fn test_master_password_can_complete() {
    let mut screen = MasterPasswordScreen::new();
    assert!(!screen.can_complete());

    screen.set_password_input("short".to_string());
    screen.set_confirm_input("short".to_string());
    screen.update_match_status();
    assert!(!screen.can_complete()); // Too short

    screen.set_password_input("longenough".to_string());
    screen.set_confirm_input("longenough".to_string());
    screen.update_match_status();
    assert!(screen.can_complete());

    screen.set_confirm_input("different".to_string());
    screen.update_match_status();
    assert!(!screen.can_complete()); // Don't match
}

#[test]
fn test_master_password_validate() {
    let mut screen = MasterPasswordScreen::new();

    assert!(screen.validate().is_err()); // Empty

    screen.set_password_input("short".to_string());
    assert!(screen.validate().is_err()); // Too short

    screen.set_password_input("longenough".to_string());
    assert!(screen.validate().is_err()); // No confirmation

    screen.set_confirm_input("different".to_string());
    screen.update_match_status();
    assert!(screen.validate().is_err()); // Don't match

    screen.set_confirm_input("longenough".to_string());
    screen.update_match_status();
    assert!(screen.validate().is_ok()); // Valid
}

#[test]
fn test_password_strength_display() {
    assert_eq!(PasswordStrength::Weak.display(), "Weak");
    assert_eq!(PasswordStrength::Medium.display(), "Medium");
    assert_eq!(PasswordStrength::Strong.display(), "Strong");
}

#[test]
fn test_password_strength_color() {
    assert_eq!(PasswordStrength::Weak.color(), Color::Red);
    assert_eq!(PasswordStrength::Medium.color(), Color::Yellow);
    assert_eq!(PasswordStrength::Strong.color(), Color::Green);
}
