use keyring_cli::crypto::{aes256gcm, argon2id, keywrap, CryptoManager};

#[test]
fn test_full_crypto_workflow() {
    // 1. Derive key from password
    let password = "secure-master-password";
    let salt = argon2id::generate_salt();
    let master_key = argon2id::derive_key(password, &salt).unwrap();
    assert_eq!(master_key.len(), 32);

    // 2. Wrap a data encryption key
    let dek = [5u8; 32];
    let master_key_array: [u8; 32] = master_key.as_slice().try_into().unwrap();
    let (wrapped_dek, nonce) = keywrap::wrap_key(&dek, &master_key_array).unwrap();

    // 3. Unwrap the key
    let unwrapped_dek = keywrap::unwrap_key(&wrapped_dek, &nonce, &master_key_array).unwrap();
    assert_eq!(dek.to_vec(), unwrapped_dek.to_vec());
}

#[test]
fn test_crypto_manager_workflow() {
    let mut crypto = CryptoManager::new();

    // Initialize with password
    crypto.initialize("test-password").unwrap();

    // Encrypt data
    let plaintext = b"sensitive-data-123";
    let (ciphertext, nonce) = crypto.encrypt(plaintext).unwrap();

    // Decrypt data
    let decrypted = crypto.decrypt(&ciphertext, &nonce).unwrap();
    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_different_passwords_different_keys() {
    let salt = argon2id::generate_salt();

    let key1 = argon2id::derive_key("password1", &salt).unwrap();
    let key2 = argon2id::derive_key("password2", &salt).unwrap();

    // Keys should be completely different
    let diff_count = key1.iter().zip(key2.iter()).filter(|(a, b)| a != b).count();

    assert!(diff_count >= 28, "Keys should differ in most bytes");
}

#[test]
fn test_encryption_integrity_tampering() {
    let plaintext = b"original-data";
    let key = [6u8; 32];

    let (mut ciphertext, nonce) = aes256gcm::encrypt(plaintext, &key).unwrap();

    // Tamper with ciphertext
    ciphertext[0] ^= 0xFF;

    // Decryption should fail
    let result = aes256gcm::decrypt(&ciphertext, &nonce, &key);
    assert!(
        result.is_err(),
        "Tampered ciphertext should fail decryption"
    );
}

#[test]
fn test_wrong_nonce_fails_decryption() {
    let plaintext = b"data-to-encrypt";
    let key = [7u8; 32];

    let (_ciphertext, nonce) = aes256gcm::encrypt(plaintext, &key).unwrap();

    // Use wrong nonce
    let mut wrong_nonce = nonce;
    wrong_nonce[0] ^= 0xFF;

    // Re-encrypt with same plaintext (different nonce is generated)
    let (ciphertext2, _) = aes256gcm::encrypt(plaintext, &key).unwrap();

    // Try to decrypt with wrong nonce
    let result = aes256gcm::decrypt(&ciphertext2, &wrong_nonce, &key);
    assert!(result.is_err(), "Wrong nonce should fail decryption");
}
