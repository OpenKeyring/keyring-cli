// src/crypto/passkey.rs
use crate::types::SensitiveString;
use anyhow::{anyhow, Result};
use bip39::{Language, Mnemonic};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// Passkey: 24-word BIP39 mnemonic as root key
#[derive(Clone, Debug)]
pub struct Passkey {
    mnemonic: Mnemonic,
}

/// Passkey-derived seed (64 bytes) - wrapped in SensitiveString for auto-zeroization
pub type PasskeySeed = SensitiveString<Vec<u8>>;

/// Wrapped passkey with encrypted seed for storage
#[derive(Clone, Debug)]
pub struct WrappedPasskey {
    pub wrapped_seed: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl Drop for WrappedPasskey {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.wrapped_seed.zeroize();
        self.nonce.zeroize();
    }
}

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
        Ok(SensitiveString::new(seed.to_vec()))
    }

    /// Validate a single BIP39 word
    pub fn is_valid_word(word: &str) -> bool {
        let word_lower = word.to_lowercase();
        Language::English.word_list().contains(&word_lower.as_str())
    }
}

/// Methods for PasskeySeed (SensitiveString<Vec<u8>>)
impl PasskeySeed {
    /// Derive root master key from Passkey seed using PBKDF2-SHA256
    ///
    /// This method derives a 32-byte root master key from the 64-byte Passkey seed
    /// using PBKDF2-HMAC-SHA256 with 600,000 iterations as recommended by OWASP.
    ///
    /// # Arguments
    /// * `salt` - 16-byte salt for key derivation
    ///
    /// # Returns
    /// 32-byte root master key
    ///
    /// # Security Note
    /// PBKDF2 with 600,000 iterations provides cross-device compatibility and
    /// is recommended by OWASP for password-based key derivation (2023).
    pub fn derive_root_master_key(&self, salt: &[u8; 16]) -> Result<[u8; 32]> {
        let seed_bytes = self.get();
        if seed_bytes.len() != 64 {
            return Err(anyhow!(
                "Passkey seed must be 64 bytes, got {}",
                seed_bytes.len()
            ));
        }

        let mut root_mk = [0u8; 32];

        // Use PBKDF2-HMAC-SHA256 with 600,000 iterations (OWASP 2023 recommendation)
        pbkdf2_hmac::<Sha256>(
            seed_bytes, // Use the full 64-byte seed as the input
            salt,
            600_000, // OWASP 2023 recommendation for PBKDF2
            &mut root_mk,
        );

        Ok(root_mk)
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
