// Legacy stub module - now uses passkey module internally
use crate::crypto::passkey::Passkey;
use anyhow::Result;

/// Generate a BIP39 mnemonic (24 words)
pub fn generate_mnemonic(word_count: usize) -> Result<String> {
    let passkey = Passkey::generate(word_count)?;
    Ok(passkey.to_words().join(" "))
}

/// Validate a BIP39 mnemonic
pub fn validate_mnemonic(mnemonic: &str) -> Result<bool> {
    let words: Vec<String> = mnemonic.split_whitespace().map(String::from).collect();
    match Passkey::from_words(&words) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic_24_words() {
        let mnemonic = generate_mnemonic(24).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 24);

        // Verify all words are valid BIP39 words
        for word in words {
            assert!(Passkey::is_valid_word(word), "Invalid word: {}", word);
        }

        // Verify the mnemonic is valid
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_generate_mnemonic_12_words() {
        let mnemonic = generate_mnemonic(12).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 12);

        // Verify all words are valid BIP39 words
        for word in words {
            assert!(Passkey::is_valid_word(word), "Invalid word: {}", word);
        }

        // Verify the mnemonic is valid
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_generate_mnemonic_different_calls_produce_different_results() {
        let mnemonic1 = generate_mnemonic(24).unwrap();
        let mnemonic2 = generate_mnemonic(24).unwrap();

        // Different calls should produce different mnemonics
        assert_ne!(mnemonic1, mnemonic2);
    }

    #[test]
    fn test_validate_mnemonic_valid_input() {
        // Generate a valid mnemonic first
        let mnemonic = generate_mnemonic(24).unwrap();
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_validate_mnemonic_invalid_words() {
        let invalid = "word1 word2 word3 this is not valid bip39";
        assert!(!validate_mnemonic(invalid).unwrap());
    }

    #[test]
    fn test_validate_mnemonic_empty_string() {
        assert!(!validate_mnemonic("").unwrap());
    }

    #[test]
    fn test_validate_mnemonic_whitespace_only() {
        assert!(!validate_mnemonic("   \n\t  ").unwrap());
    }

    #[test]
    fn test_validate_mnemonic_wrong_word_count() {
        // Only 3 words - not enough for valid BIP39
        let too_few = "abandon abandon abandon";
        assert!(!validate_mnemonic(too_few).unwrap());
    }

    #[test]
    fn test_generate_mnemonic_invalid_word_count() {
        // 11 words is not a valid BIP39 word count
        let result = generate_mnemonic(11);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_mnemonic_15_words() {
        let mnemonic = generate_mnemonic(15).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 15);
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_generate_mnemonic_18_words() {
        let mnemonic = generate_mnemonic(18).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 18);
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_generate_mnemonic_21_words() {
        let mnemonic = generate_mnemonic(21).unwrap();
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert_eq!(words.len(), 21);
        assert!(validate_mnemonic(&mnemonic).unwrap());
    }

    #[test]
    fn test_validate_mnemonic_with_extra_whitespace() {
        let mnemonic = generate_mnemonic(12).unwrap();
        // Add extra whitespace
        let with_extra = format!("  {}  \n  ", mnemonic);
        // Should still validate as split_whitespace handles it
        assert!(validate_mnemonic(&with_extra).unwrap());
    }

    #[test]
    fn test_validate_mnemonic_checksum_validation() {
        // Use a known valid BIP39 test vector (from BIP39 spec)
        // 128 bits entropy: 0000000000000000000000000000000000000000000000000000000000000000
        let valid_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert!(validate_mnemonic(valid_mnemonic).unwrap());

        // Test 1: Corrupt the checksum by changing the last word
        // "abandon" (word 0) vs "about" (word 127) - different 11-bit values
        // The checksum (last 4 bits) will no longer match SHA256(entire entropy)
        let invalid_last_word = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(!validate_mnemonic(invalid_last_word).unwrap(),
            "Mnemonic with corrupted last word (wrong checksum) should be invalid");

        // Test 2: Change a middle word to invalidate both entropy and checksum
        // Word 5 changed from "abandon" (0) to "ability" (7)
        // This corrupts the entropy bits, making the checksum invalid
        let invalid_middle = "abandon abandon abandon abandon abandon ability abandon abandon abandon abandon abandon about";
        assert!(!validate_mnemonic(invalid_middle).unwrap(),
            "Mnemonic with corrupted middle word should be invalid");

        // Test 3: Use wrong word count (only 11 words instead of 12)
        let wrong_count = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(!validate_mnemonic(wrong_count).unwrap(),
            "Mnemonic with wrong word count should be invalid");

        // Test 4: Words in wrong order (scrambled)
        let scrambled = "about abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(!validate_mnemonic(scrambled).unwrap(),
            "Mnemonic with scrambled word order should be invalid");
    }
}
