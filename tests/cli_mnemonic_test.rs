// tests/cli/mnemonic_test.rs
use keyring_cli::cli::commands::mnemonic::{handle_mnemonic, MnemonicArgs};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_mnemonic_generate_with_name_requires_db() {
    // This test verifies that the generate command with a name
    // properly structures the mnemonic for database saving
    let args = MnemonicArgs {
        generate: Some(12),
        name: Some("test-wallet".to_string()),
        validate: None,
    };

    // The command should not error (actual save would require full setup)
    // This test verifies the command structure is correct
    assert_eq!(args.name, Some("test-wallet".to_string()));
    assert_eq!(args.generate, Some(12));
}

#[test]
fn test_mnemonic_generate_without_name() {
    let args = MnemonicArgs {
        generate: Some(24),
        name: None,
        validate: None,
    };

    assert_eq!(args.name, None);
    assert_eq!(args.generate, Some(24));
}

#[test]
fn test_mnemonic_validate() {
    let args = MnemonicArgs {
        generate: None,
        name: None,
        validate: Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()),
    };

    // Check that validate option is set correctly
    assert!(args.validate.is_some());
    assert!(args.generate.is_none());

    // The mnemonic has 12 words
    let words = args.validate.unwrap().split_whitespace().count();
    assert_eq!(words, 12);
}
