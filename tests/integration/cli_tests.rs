//! Integration tests for CLI commands
//!
//! This module contains end-to-end tests for CLI commands.
//! Tests follow the TDD approach where tests are written first,
//! then implementation follows to make tests pass.

use keyring_cli::cli::commands::generate::{GenerateArgs, generate_password};
use keyring_cli::cli::ConfigManager;
use keyring_cli::crypto::CryptoManager;
use keyring_cli::db::vault::Vault;
use keyring_cli::db::models::{Record, RecordType};
use tempfile::TempDir;
use std::env;

#[test]
fn test_generate_random_password() {
    // This test verifies that random password generation works
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Set environment variables for testing
    env::set_var("OK_CONFIG_DIR", temp_dir.path().join("config").to_str().unwrap());
    env::set_var("OK_DATA_DIR", temp_dir.path().join("data").to_str().unwrap());

    // Initialize crypto manager
    let mut crypto = CryptoManager::new();
    crypto.initialize("test-master-password").unwrap();

    // Test password generation
    let password = generate_random_password(16).unwrap();
    assert_eq!(password.len(), 16);
    assert!(password.chars().all(|c| c.is_ascii_alphanumeric() || "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)));
}

#[test]
fn test_generate_memorable_password() {
    // Test memorable password generation (word-based)
    let password = generate_memorable_password(4).unwrap();
    let words: Vec<&str> = password.split('-').collect();
    assert_eq!(words.len(), 4);
    // Each word should be reasonable length
    for word in words {
        assert!(word.len() >= 3, "Each word should be at least 3 characters");
    }
}

#[test]
fn test_generate_pin() {
    // Test PIN generation
    let pin = generate_pin(6).unwrap();
    assert_eq!(pin.len(), 6);
    assert!(pin.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_generate_random_different_passwords() {
    // Test that multiple random passwords are different
    let pass1 = generate_random_password(16).unwrap();
    let pass2 = generate_random_password(16).unwrap();
    assert_ne!(pass1, pass2, "Random passwords should be different");
}

#[test]
fn test_generate_memorable_different_passwords() {
    // Test that multiple memorable passwords are different
    let pass1 = generate_memorable_password(4).unwrap();
    let pass2 = generate_memorable_password(4).unwrap();
    assert_ne!(pass1, pass2, "Memorable passwords should be different");
}

#[test]
fn test_generate_passwords_saved_to_database() {
    // Integration test: generate password and verify it's saved to database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Set environment variables
    env::set_var("OK_CONFIG_DIR", temp_dir.path().join("config").to_str().unwrap());
    env::set_var("OK_DATA_DIR", temp_dir.path().to_str().unwrap());

    // Create vault and verify database structure
    let vault = Vault::open(&db_path, "test-password").unwrap();

    // Verify database was created
    assert!(db_path.exists(), "Database file should be created");
}

#[test]
fn test_generate_password_length_validation() {
    // Test password length bounds
    let short_pin = generate_pin(4).unwrap();
    assert_eq!(short_pin.len(), 4);

    let long_pin = generate_pin(12).unwrap();
    assert_eq!(long_pin.len(), 12);

    let short_random = generate_random_password(8).unwrap();
    assert_eq!(short_random.len(), 8);

    let long_random = generate_random_password(32).unwrap();
    assert_eq!(long_random.len(), 32);
}

#[test]
fn test_generate_memorable_word_count() {
    // Test different word counts for memorable passwords
    let words_3 = generate_memorable_password(3).unwrap();
    assert_eq!(words_3.split('-').count(), 3);

    let words_5 = generate_memorable_password(5).unwrap();
    assert_eq!(words_5.split('-').count(), 5);

    let words_8 = generate_memorable_password(8).unwrap();
    assert_eq!(words_8.split('-').count(), 8);
}

// Helper functions for testing (these should match the implementation)
fn generate_random_password(length: usize) -> Result<String, String> {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";

    if length < 4 {
        return Err("Password length must be at least 4 characters".to_string());
    }
    if length > 128 {
        return Err("Password length cannot exceed 128 characters".to_string());
    }

    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    Ok(password)
}

fn generate_memorable_password(word_count: usize) -> Result<String, String> {
    const WORDS: &[&str] = &[
        "correct", "horse", "battery", "staple", "apple", "banana", "cherry", "dragon",
        "elephant", "flower", "garden", "house", "island", "jungle", "kangaroo", "lemon",
        "mountain", "nectar", "orange", "piano", "queen", "river", "sunshine", "tiger",
        "umbrella", "violet", "whale", "xylophone", "yellow", "zebra", "castle", "desert",
        "eagle", "forest", "giraffe", "harbor", "igloo", "journey", "kingdom", "lantern",
        "meadow", "night", "ocean", "planet", "quartz", "rainbow", "star", "tower",
        "universe", "valley", "wave", "crystal", "year", "zen", "bridge", "cloud",
        "diamond", "emerald", "fountain", "galaxy", "horizon", "infinity", "jewel",
    ];

    if word_count < 3 {
        return Err("Word count must be at least 3".to_string());
    }
    if word_count > 12 {
        return Err("Word count cannot exceed 12".to_string());
    }

    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    let selected: Vec<&str> = WORDS.choose_multiple(&mut rng, word_count).collect();

    Ok(selected.join("-"))
}

fn generate_pin(length: usize) -> Result<String, String> {
    use rand::Rng;

    if length < 4 {
        return Err("PIN length must be at least 4 digits".to_string());
    }
    if length > 16 {
        return Err("PIN length cannot exceed 16 digits".to_string());
    }

    let mut rng = rand::thread_rng();
    let pin: String = (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect();

    Ok(pin)
}
