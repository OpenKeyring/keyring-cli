//! BIP39 mnemonic for recovery key

use anyhow::Result;

/// Generate a BIP39 mnemonic phrase (12 or 24 words)
pub fn generate_mnemonic(word_count: usize) -> Result<String> {
    match word_count {
        12 | 24 => Ok(format!("stub-mnemonic-{}-words", word_count)),
        _ => anyhow::bail!("word_count must be 12 or 24"),
    }
}

/// Validate a BIP39 mnemonic phrase
pub fn validate_mnemonic(mnemonic: &str) -> Result<bool> {
    Ok(mnemonic.starts_with("stub-")) // Stub validation
}

/// Convert mnemonic to entropy bytes
pub fn mnemonic_to_entropy(_mnemonic: &str) -> Result<Vec<u8>> {
    Ok(vec![0u8; 32])
}
