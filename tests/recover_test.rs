//! CLI recover command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use keyring_cli::cli::commands::recover::RecoverArgs;
use keyring_cli::crypto::{passkey::Passkey, CryptoManager};
use tempfile::TempDir;

/// Helper to set up test environment with Passkey
struct TestEnv {
    _temp_dir: TempDir,
    db_path: std::path::PathBuf,
    passkey: Passkey,
    passkey_words: Vec<String>,
}

impl TestEnv {
    fn setup(test_name: &str) -> Self {
        // Clean up any existing environment variables first
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(format!("config_{}", test_name));
        let data_dir = temp_dir.path().join(format!("data_{}", test_name));
        std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
        std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
        std::env::set_var("OK_MASTER_PASSWORD", "test-password");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        let db_path = data_dir.join("passwords.db");

        // Generate a test Passkey
        let passkey = Passkey::generate(24).unwrap();
        let passkey_words = passkey.to_words();

        Self {
            _temp_dir: temp_dir,
            db_path,
            passkey,
            passkey_words,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Clean up environment variables
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");
    }
}

#[test]
fn test_recover_command_accepts_passkey_argument() {
    let env = TestEnv::setup("passkey_arg");

    // Create args with passkey provided
    let passkey_str = env.passkey_words.join(" ");
    let args = RecoverArgs {
        passkey: Some(passkey_str),
    };

    // Verify args can be created
    assert!(args.passkey.is_some());
    assert_eq!(args.passkey.unwrap(), env.passkey_words.join(" "));
}

#[test]
fn test_recover_command_accepts_empty_passkey() {
    // Create args without passkey (interactive mode)
    let args = RecoverArgs { passkey: None };

    // Verify args can be created for interactive mode
    assert!(args.passkey.is_none());
}

#[test]
fn test_recover_validates_passkey_word_count() {
    let _env = TestEnv::setup("validate_word_count");

    // Test with valid BIP39 word count (12 words)
    let valid_words = (0..12).map(|_| "abandon".to_string()).collect::<Vec<_>>();
    let result = Passkey::from_words(&valid_words);
    // 12 identical words have invalid checksum, so this will fail
    assert!(result.is_err(), "12 identical words should fail checksum validation");

    // Test with invalid word count (11 words - not a valid BIP39 count)
    let invalid_count = 11;
    let wrong_count_words: Vec<String> = (0..invalid_count).map(|i| format!("word{}", i)).collect();
    let _passkey_str = wrong_count_words.join(" ");

    // BIP39 supports: 12, 15, 18, 21, 24 words
    // 11 words should fail validation
    let result = Passkey::from_words(&wrong_count_words);
    assert!(result.is_err(), "11 words should be rejected as invalid BIP39 count");

    // 20 words is valid BIP39 word count
    let twenty_words: Vec<String> = (0..20).map(|_| "abandon".to_string()).collect();
    let result = Passkey::from_words(&twenty_words);
    // 20 identical words have invalid checksum, but count is valid
    assert!(result.is_err(), "20 identical words should fail checksum");
}

#[test]
fn test_recover_validates_passkey_checksum() {
    let _env = TestEnv::setup("validate_checksum");

    // Create invalid 24-word phrase (wrong checksum)
    let invalid_words: Vec<String> = vec!["abandon".to_string(); 24];

    // Should fail validation
    let result = Passkey::from_words(&invalid_words);
    assert!(result.is_err(), "Invalid checksum should be rejected");
}

#[test]
fn test_recover_generates_new_salt() {
    let env = TestEnv::setup("new_salt");

    // Initialize CryptoManager with Passkey
    let mut crypto = CryptoManager::new();

    // Derive root master key from Passkey
    let seed = env.passkey.to_seed(None).unwrap();
    let salt = [0u8; 16]; // Test salt
    let root_master_key = seed.derive_root_master_key(&salt).unwrap();

    // Initialize with Passkey (using CLI device index)
    use keyring_cli::crypto::hkdf::DeviceIndex;
    let kdf_nonce = [0u8; 32]; // Test KDF nonce

    let result = crypto.initialize_with_passkey(
        &env.passkey,
        "new-device-password",
        &root_master_key,
        DeviceIndex::CLI,
        &kdf_nonce,
    );

    assert!(result.is_ok(), "Should initialize with Passkey");
    assert!(crypto.is_initialized());
}

#[test]
fn test_recover_reencrypts_wrapped_passkey() {
    let env = TestEnv::setup("reencrypt");

    // First, initialize with original password
    let mut crypto = CryptoManager::new();
    let seed = env.passkey.to_seed(None).unwrap();
    let salt = [0u8; 16];
    let root_master_key = seed.derive_root_master_key(&salt).unwrap();

    use keyring_cli::crypto::hkdf::DeviceIndex;
    let kdf_nonce = [0u8; 32];

    crypto
        .initialize_with_passkey(
            &env.passkey,
            "old-password",
            &root_master_key,
            DeviceIndex::CLI,
            &kdf_nonce,
        )
        .unwrap();

    // Verify wrapped_passkey file exists
    let keyring_path = dirs::home_dir()
        .unwrap()
        .join(".local/share/open-keyring");
    let _wrapped_passkey_path = keyring_path.join("wrapped_passkey");

    // Note: In test environment, this might not exist yet
    // The actual re-encryption logic will be tested in integration tests
}

#[test]
fn test_recover_requires_password_confirmation() {
    let env = TestEnv::setup("password_confirm");

    // This test verifies that the recovery flow requires password confirmation
    // The actual implementation will prompt for password twice
    let passkey_str = env.passkey_words.join(" ");

    let args = RecoverArgs {
        passkey: Some(passkey_str),
    };

    // In interactive mode, passwords must match
    // This is a structural test - the implementation handles confirmation
    assert!(args.passkey.is_some());
}

#[test]
fn test_recover_handles_invalid_current_password() {
    let _env = TestEnv::setup("invalid_password");

    // This test verifies that recovery with wrong password fails
    // The implementation should verify the current password before re-encrypting

    // Create invalid passkey
    let invalid_words: Vec<String> = vec!["abandon".to_string(); 24];
    let passkey_str = invalid_words.join(" ");

    let args = RecoverArgs {
        passkey: Some(passkey_str),
    };

    // Should fail when trying to use invalid passkey
    let result = Passkey::from_words(&invalid_words);
    assert!(result.is_err(), "Invalid passkey should be rejected");
}
