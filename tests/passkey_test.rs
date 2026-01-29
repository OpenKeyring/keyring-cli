// tests/passkey_test.rs
use keyring_cli::crypto::passkey::Passkey;

#[test]
fn test_generate_passkey_24_words() {
    let passkey = Passkey::generate(24).unwrap();
    let words = passkey.to_words();
    assert_eq!(words.len(), 24);

    // Verify all words are valid BIP39 words
    for word in &words {
        assert!(Passkey::is_valid_word(word));
    }
}

#[test]
fn test_passkey_to_seed() {
    let passkey = Passkey::generate(24).unwrap();
    let seed = passkey.to_seed(None).unwrap();
    assert_eq!(seed.0.len(), 64); // BIP39 seed is 64 bytes
}

#[test]
fn test_passkey_from_words() {
    let original = Passkey::generate(24).unwrap();
    let words = original.to_words();

    let restored = Passkey::from_words(&words).unwrap();
    assert_eq!(original.to_seed(None).unwrap().0, restored.to_seed(None).unwrap().0);
}

#[test]
fn test_passkey_with_optional_passphrase() {
    let passkey = Passkey::generate(12).unwrap();
    let seed_no_passphrase = passkey.to_seed(None).unwrap();
    let seed_with_passphrase = passkey.to_seed(Some("test-passphrase")).unwrap();

    // Different passphrases should produce different seeds
    assert_ne!(seed_no_passphrase.0, seed_with_passphrase.0);
}
