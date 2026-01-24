use keyring_cli::crypto::{aes256gcm, argon2id};

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
