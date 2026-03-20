//! Tests for generate command
//!
//! Unit tests for password generation functionality.

use super::*;

fn create_test_args() -> args::NewArgs {
    args::NewArgs {
        name: "test".to_string(),
        length: 16,
        memorable: false,
        pin: false,
        numbers: false,
        symbols: false,
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        sync: false,
        words: 4,
        copy: false,
    }
}

#[test]
fn test_generate_args_validation_empty_name() {
    let mut args = create_test_args();
    args.name = String::new();
    assert!(args.validate().is_err());
}

#[test]
fn test_generate_args_validation_invalid_length() {
    let mut args = create_test_args();
    args.length = 2; // Too short
    assert!(args.validate().is_err());
}

#[test]
fn test_generate_args_get_password_type() {
    let args = create_test_args();
    assert_eq!(
        args.get_password_type().unwrap(),
        args::PasswordType::Random
    );
}

#[test]
fn test_generate_args_memorable_flag() {
    let mut args = create_test_args();
    args.memorable = true;
    assert_eq!(
        args.get_password_type().unwrap(),
        args::PasswordType::Memorable
    );
}

#[test]
fn test_generate_args_pin_flag() {
    let mut args = create_test_args();
    args.pin = true;
    assert_eq!(args.get_password_type().unwrap(), args::PasswordType::Pin);
}

#[test]
fn test_generate_args_pin_priority_over_memorable() {
    let mut args = create_test_args();
    args.pin = true;
    args.memorable = true;
    // PIN should take priority
    assert_eq!(args.get_password_type().unwrap(), args::PasswordType::Pin);
}

#[test]
fn test_password_description() {
    assert_eq!(
        args::get_password_description(args::PasswordType::Random),
        "Random password with special characters"
    );
    assert_eq!(
        args::get_password_description(args::PasswordType::Memorable),
        "Memorable word-based passphrase"
    );
    assert_eq!(
        args::get_password_description(args::PasswordType::Pin),
        "Numeric PIN code"
    );
}

#[test]
fn test_generate_random_password_length() {
    let password = generators::generate_random(16, true, true).unwrap();
    assert_eq!(password.len(), 16);
}

#[test]
fn test_generate_random_password_no_ambiguous_chars() {
    let password = generators::generate_random(100, true, true).unwrap();
    // Should not contain ambiguous characters
    assert!(!password.contains('0'));
    assert!(!password.contains('1'));
    assert!(!password.contains('O'));
    assert!(!password.contains('I'));
    assert!(!password.contains('o'));
    assert!(!password.contains('l'));
}

#[test]
fn test_generate_random_password_without_numbers_or_symbols() {
    let password = generators::generate_random(16, false, false).unwrap();
    // Should only contain letters (no numbers or symbols)
    assert!(password.chars().all(|c| c.is_alphabetic()));
}

#[test]
fn test_generate_random_password_too_short() {
    let result = generators::generate_random(3, true, true);
    assert!(result.is_err());
}

#[test]
fn test_generate_random_password_too_long() {
    let result = generators::generate_random(129, true, true);
    assert!(result.is_err());
}

#[test]
fn test_generate_memorable_password_word_count() {
    let password = generators::generate_memorable(4).unwrap();
    // Should have 4 words separated by hyphens
    let parts: Vec<&str> = password.split('-').collect();
    assert_eq!(parts.len(), 4);
}

#[test]
fn test_generate_memorable_password_capitalization() {
    let password = generators::generate_memorable(4).unwrap();
    // Each word should start with uppercase
    for word in password.split('-') {
        assert!(word.chars().next().unwrap().is_uppercase());
    }
}

#[test]
fn test_generate_memorable_password_too_few_words() {
    let result = generators::generate_memorable(2);
    assert!(result.is_err());
}

#[test]
fn test_generate_memorable_password_too_many_words() {
    let result = generators::generate_memorable(13);
    assert!(result.is_err());
}

#[test]
fn test_generate_pin_length() {
    let pin = generators::generate_pin(6).unwrap();
    assert_eq!(pin.len(), 6);
}

#[test]
fn test_generate_pin_only_2_to_9() {
    let pin = generators::generate_pin(16).unwrap();
    // Should only contain digits 2-9
    assert!(pin
        .chars()
        .all(|c| c.is_ascii_digit() && ('2'..='9').contains(&c)));
    // Should not contain 0 or 1
    assert!(!pin.contains('0'));
    assert!(!pin.contains('1'));
}

#[test]
fn test_generate_pin_too_short() {
    let result = generators::generate_pin(3);
    assert!(result.is_err());
}

#[test]
fn test_generate_pin_too_long() {
    let result = generators::generate_pin(17);
    assert!(result.is_err());
}
