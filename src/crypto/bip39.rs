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
