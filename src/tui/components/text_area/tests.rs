//! Tests for TextArea component
//!
//! Unit tests for the text area component.

use super::*;
use crate::tui::traits::{BuiltinValidator, ValidationTrigger};

#[test]
fn test_text_area_creation() {
    let textarea = TextArea::new("placeholder");
    assert_eq!(textarea.placeholder(), "placeholder");
    assert_eq!(textarea.text(), "");
}

#[test]
fn test_text_area_set_text() {
    let mut textarea = TextArea::new("placeholder");
    textarea.set_text("Hello\nWorld\nTest".to_string());

    assert_eq!(textarea.text(), "Hello\nWorld\nTest");
    assert_eq!(textarea.lines.len(), 3);
}

#[test]
fn test_text_area_insert_char() {
    let mut textarea = TextArea::new("placeholder");
    textarea.insert_char('H');
    textarea.insert_char('i');

    assert_eq!(textarea.text(), "Hi");
}

#[test]
fn test_text_area_newline() {
    let mut textarea = TextArea::new("placeholder");
    textarea.insert_char('H');
    textarea.insert_char('i');
    textarea.insert_newline();
    textarea.insert_char('Y');
    textarea.insert_char('o');
    textarea.insert_char('u');

    assert_eq!(textarea.text(), "Hi\nYou");
}

#[test]
fn test_text_area_cursor_movement() {
    let mut textarea = TextArea::new("placeholder");
    textarea.set_text("Line1\nLine2\nLine3".to_string());

    // Start at end of last line
    assert_eq!(textarea.cursor_row, 2);
    assert_eq!(textarea.cursor_col, 5); // length of "Line3"

    // Move up
    textarea.move_up();
    assert_eq!(textarea.cursor_row, 1);
    assert_eq!(textarea.cursor_col, 5); // Should keep column position

    // Move down
    textarea.move_down();
    assert_eq!(textarea.cursor_row, 2);
}

#[test]
fn test_text_area_backspace() {
    let mut textarea = TextArea::new("placeholder");
    textarea.set_text("Hello World".to_string());
    textarea.cursor_col = 6; // Position at "W"

    textarea.backspace(); // Remove space

    assert_eq!(textarea.text(), "HelloWorld");
}

#[test]
fn test_text_area_clear() {
    let mut textarea = TextArea::new("placeholder");
    textarea.set_text("Some text\nMore text".to_string());

    textarea.clear();

    assert_eq!(textarea.text(), "");
    assert_eq!(textarea.lines.len(), 1);
    assert_eq!(textarea.lines[0], "");
}

#[test]
fn test_text_area_validation() {
    let validation = FieldValidation::new()
        .with_validator(BuiltinValidator::Required)
        .with_trigger(ValidationTrigger::OnBlur);

    let mut textarea = TextArea::new("placeholder").with_validation(validation);

    textarea.set_text("".to_string()); // Empty text should fail validation
    let result = textarea.validate();

    assert!(!result.is_valid);
    assert!(result.has_errors());
}

#[test]
fn test_text_area_max_lines() {
    let mut textarea = TextArea::new("placeholder").with_max_lines(2);

    textarea.insert_char('A');
    textarea.insert_newline();
    textarea.insert_char('B');
    textarea.insert_newline(); // This should exceed max lines
    textarea.insert_char('C');

    // Get the text first, then check
    let text = textarea.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() <= 2);
}
