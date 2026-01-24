use keyring_cli::crypto::{aes256gcm, argon2id, CryptoManager};

#[test]
fn test_argon2id_derive_key() {
    let password = "test-password-12345";
    let salt = b"test-salt-16-bye";
    let key = argon2id::derive_key(password, salt).unwrap();
    assert_eq!(key.len(), 32); // 256 bits
}

#[test]
fn test_aes256gcm_encrypt_decrypt() {
    let plaintext = b"secret message";
    let key = [0u8; 32]; // Test key
    let (ciphertext, nonce) = aes256gcm::encrypt(plaintext, &key).unwrap();
    let decrypted = aes256gcm::decrypt(&ciphertext, &nonce, &key).unwrap();
    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_crypto_manager_initialize_with_key() {
    let mut crypto = CryptoManager::new();
    let key = [7u8; 32];
    crypto.initialize_with_key(key);

    let (cipher, nonce) = crypto.encrypt(b"payload").unwrap();
    let decrypted = crypto.decrypt(&cipher, &nonce).unwrap();
    assert_eq!(decrypted, b"payload");
}
