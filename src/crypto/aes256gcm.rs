use aes_gcm::{
    aead::{Aead, AeadCore, AeadInPlace, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::Result;

/// Encrypted data with nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

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

/// Encrypt data with Additional Authenticated Data (AAD)
pub fn encrypt_with_aad(plaintext: &[u8], aad: &[u8], key: &[u8; 32]) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Allocate buffer with space for tag (16 bytes overhead)
    let mut buffer = Vec::with_capacity(plaintext.len() + 16);
    buffer.extend_from_slice(plaintext);

    cipher
        .encrypt_in_place(&nonce, aad, &mut buffer)
        .map_err(|e| anyhow::anyhow!("AES-256-GCM encryption failed: {}", e))?;

    Ok(EncryptedData {
        ciphertext: buffer,
        nonce: nonce.into(),
    })
}

/// Decrypt data with Additional Authenticated Data (AAD)
pub fn decrypt_with_aad(encrypted: &EncryptedData, aad: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from(encrypted.nonce);

    let mut buffer = encrypted.ciphertext.clone();

    cipher
        .decrypt_in_place(&nonce, aad, &mut buffer)
        .map_err(|e| anyhow::anyhow!("AES-256-GCM decryption failed: {}", e))?;

    Ok(buffer)
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

    #[test]
    fn test_encrypt_with_aad() {
        let plaintext = b"sensitive data";
        let aad = b"additional-authenticated-data";
        let key = [2u8; 32];

        let result = encrypt_with_aad(plaintext, aad, &key).unwrap();
        assert!(result.ciphertext.len() > 0);
        assert_eq!(result.nonce.len(), 12);
    }

    #[test]
    fn test_encrypt_decrypt_aad_roundtrip() {
        let plaintext = b"secret message";
        let aad = b"context-data";
        let key = [3u8; 32];

        let encrypted = encrypt_with_aad(plaintext, aad, &key).unwrap();
        let decrypted = decrypt_with_aad(&encrypted, aad, &key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
