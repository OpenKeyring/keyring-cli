use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;

/// Encrypt data using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - Data to encrypt
/// * `key` - 32-byte (256-bit) encryption key
///
/// # Returns
/// Tuple of (ciphertext, nonce)
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12])> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("AES-256-GCM encryption failed: {}", e))?;

    // Extract nonce bytes
    let nonce_bytes: [u8; 12] = nonce.into();

    Ok((ciphertext, nonce_bytes))
}

/// Decrypt data using AES-256-GCM
///
/// # Arguments
/// * `ciphertext` - Encrypted data
/// * `nonce` - 12-byte nonce used for encryption
/// * `key` - 32-byte (256-bit) decryption key
///
/// # Returns
/// Decrypted plaintext
pub fn decrypt(ciphertext: &[u8], nonce: &[u8; 12], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from(*nonce);

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("AES-256-GCM decryption failed: {}", e))?;

    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, World!";
        let key = [0u8; 32]; // Test key

        let (ciphertext, nonce) = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &nonce, &key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let plaintext = b"Test data";
        let key = [1u8; 32];

        let (ciphertext1, nonce1) = encrypt(plaintext, &key).unwrap();
        let (ciphertext2, nonce2) = encrypt(plaintext, &key).unwrap();

        // Nonces should be different (randomly generated)
        assert_ne!(nonce1, nonce2);
        // Ciphertexts should also differ due to different nonces
        assert_ne!(ciphertext1, ciphertext2);
    }
}
