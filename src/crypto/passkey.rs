// src/crypto/passkey.rs
use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use zeroize::ZeroizeOnDrop;

/// Passkey: 24-word BIP39 mnemonic as root key
#[derive(Clone, Debug)]
pub struct Passkey {
    mnemonic: Mnemonic,
}

/// Passkey-derived seed (64 bytes)
#[derive(ZeroizeOnDrop)]
pub struct PasskeySeed(pub [u8; 64]);

impl Passkey {
    /// Generate a new Passkey with specified word count (12, 15, 18, 21, or 24)
    pub fn generate(word_count: usize) -> Result<Self> {
        if ![12, 15, 18, 21, 24].contains(&word_count) {
            return Err(anyhow!("Invalid word count: {}", word_count));
        }

        let mnemonic = Mnemonic::generate(word_count)
            .map_err(|e| anyhow!("Failed to generate Passkey: {}", e))?;

        Ok(Self { mnemonic })
    }

    /// Create Passkey from word list
    pub fn from_words(words: &[String]) -> Result<Self> {
        if words.is_empty() {
            return Err(anyhow!("Word list cannot be empty"));
        }

        let phrase = words.join(" ");
        let mnemonic = Mnemonic::parse(&phrase).map_err(|e| anyhow!("Invalid Passkey: {}", e))?;

        Ok(Self { mnemonic })
    }

    /// Get word list
    pub fn to_words(&self) -> Vec<String> {
        self.mnemonic.words().map(String::from).collect()
    }

    /// Convert to seed (64 bytes) with optional passphrase
    pub fn to_seed(&self, passphrase: Option<&str>) -> Result<PasskeySeed> {
        let seed = self.mnemonic.to_seed_normalized(passphrase.unwrap_or(""));
        Ok(PasskeySeed(seed))
    }

    /// Validate a single BIP39 word
    pub fn is_valid_word(word: &str) -> bool {
        let word_lower = word.to_lowercase();
        Language::English.word_list().contains(&word_lower.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_basic() {
        let passkey = Passkey::generate(24).unwrap();
        assert_eq!(passkey.to_words().len(), 24);
    }
}
