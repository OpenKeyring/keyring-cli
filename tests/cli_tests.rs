//! Integration tests for CLI commands
//!
//! This module contains end-to-end tests for CLI commands.
//! Tests follow the TDD approach where tests are written first,
//! then implementation follows to make tests pass.

use keyring_cli::cli::commands::generate::{
    generate_memorable, generate_password, generate_pin, generate_random, GenerateArgs,
    PasswordType,
};

#[tokio::test]
async fn test_generate_random_password() {
    // Test generating a random password
    let args = GenerateArgs {
        name: "test-password".to_string(),
        length: 16,
        numbers: true,
        symbols: true,
        memorable: false,
        words: 4,
        pin: false,
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        copy: false,
        sync: false,
    };

    let result = generate_password(args).await;
    assert!(result.is_ok(), "Password generation should succeed");
}

#[tokio::test]
async fn test_generate_memorable_password() {
    let args = GenerateArgs {
        name: "test-memorable".to_string(),
        length: 16,
        numbers: false,
        symbols: false,
        memorable: true,
        words: 4,
        pin: false,
        username: Some("testuser".to_string()),
        url: Some("https://example.com".to_string()),
        notes: Some("Test notes".to_string()),
        tags: vec!["test".to_string(), "integration".to_string()],
        copy: false,
        sync: false,
    };

    let result = generate_password(args).await;
    assert!(
        result.is_ok(),
        "Memorable password generation should succeed"
    );
}

#[tokio::test]
async fn test_generate_pin() {
    let args = GenerateArgs {
        name: "test-pin".to_string(),
        length: 6,
        numbers: false,
        symbols: false,
        memorable: false,
        words: 4,
        pin: true,
        username: None,
        url: None,
        notes: None,
        tags: vec![],
        copy: false,
        sync: false,
    };

    let result = generate_password(args).await;
    assert!(result.is_ok(), "PIN generation should succeed");
}

#[test]
fn test_generate_random_password_contains_numbers() {
    // When numbers=true, password should contain at least one digit
    let password = generate_random(16, true, false).unwrap();
    assert_eq!(password.len(), 16, "Password should be 16 characters");
    assert!(
        password.chars().any(|c| c.is_ascii_digit()),
        "Password with numbers=true should contain at least one digit"
    );
}

#[test]
fn test_generate_random_password_contains_symbols() {
    // When symbols=true, password should contain at least one symbol
    let password = generate_random(16, false, true).unwrap();
    assert_eq!(password.len(), 16, "Password should be 16 characters");
    assert!(
        password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)),
        "Password with symbols=true should contain at least one symbol"
    );
}

#[test]
fn test_generate_random_password_contains_both() {
    // When both numbers and symbols are true, password should contain both
    let password = generate_random(20, true, true).unwrap();
    assert_eq!(password.len(), 20, "Password should be 20 characters");
    assert!(
        password.chars().any(|c| c.is_ascii_digit()),
        "Password should contain at least one digit"
    );
    assert!(
        password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)),
        "Password should contain at least one symbol"
    );
}

#[test]
fn test_generate_memorable_password_format() {
    let password = generate_memorable(4).unwrap();
    // Should have 4 words separated by hyphens
    let parts: Vec<&str> = password.split('-').collect();
    assert_eq!(parts.len(), 4, "Should have 4 words");
    // Each word should start with uppercase
    for word in parts {
        assert!(
            word.chars().next().unwrap().is_uppercase(),
            "Each word should start with uppercase"
        );
    }
}

#[test]
fn test_generate_pin_format() {
    let pin = generate_pin(8).unwrap();
    assert_eq!(pin.len(), 8, "PIN should be 8 digits");
    assert!(
        pin.chars().all(|c| c.is_ascii_digit()),
        "PIN should only contain digits"
    );
    // Should only use digits 2-9 (no 0 or 1)
    assert!(
        pin.chars().all(|c| c >= '2' && c <= '9'),
        "PIN should only use digits 2-9"
    );
}
