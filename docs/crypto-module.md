# Crypto Module Documentation

## Overview

The crypto module provides secure cryptographic primitives for the OpenKeyring password manager.

## Components

### Argon2id Key Derivation

- **Dynamic parameter selection** based on device capability
- **Memory-hard** hashing to prevent brute force attacks
- **Configurable** time, memory, and parallelism parameters

#### Usage

```rust
use keyring_cli::crypto::argon2id;

// Derive key with default parameters
let salt = argon2id::generate_salt();
let key = argon2id::derive_key("password", &salt)?;

// Derive key with custom parameters
let params = argon2id::Argon2Params {
    time: 3,
    memory: 64,
    parallelism: 2,
};
let key = argon2id::derive_key_with_params("password", &salt, params)?;

// Hash and verify passwords
let hash = argon2id::hash_password("password")?;
let valid = argon2id::verify_password("password", &hash)?;
```

### AES-256-GCM Encryption

- **Authenticated encryption** with AEAD
- **Unique nonce** per encryption operation
- **Additional Authenticated Data (AAD)** support

#### Usage

```rust
use keyring_cli::crypto::aes256gcm;

// Basic encrypt/decrypt
let plaintext = b"sensitive data";
let key = [0u8; 32]; // Must be exactly 32 bytes

let (ciphertext, nonce) = aes256gcm::encrypt(plaintext, &key)?;
let decrypted = aes256gcm::decrypt(&ciphertext, &nonce, &key)?;

// With AAD
let aad = b"additional-context";
let encrypted = aes256gcm::encrypt_with_aad(plaintext, aad, &key)?;
let decrypted = aes256gcm::decrypt_with_aad(&encrypted, aad, &key)?;
```

### Key Wrapping

Securely wrap keys for storage or transmission.

#### Usage

```rust
use keyring_cli::crypto::keywrap;

let wrapping_key = [1u8; 32];
let key_to_wrap = [2u8; 32];

// Wrap
let (wrapped, nonce) = keywrap::wrap_key(&key_to_wrap, &wrapping_key)?;

// Unwrap
let unwrapped = keywrap::unwrap_key(&wrapped, &nonce, &wrapping_key)?;
```

### CryptoManager

High-level API combining key derivation and encryption.

#### Usage

```rust
use keyring_cli::crypto::CryptoManager;

let mut crypto = CryptoManager::new();
crypto.initialize("master-password")?;

// Encrypt/decrypt with master key
let (ciphertext, nonce) = crypto.encrypt(b"data")?;
let decrypted = crypto.decrypt(&ciphertext, &nonce)?;

// Derive sub-keys for different purposes
let enc_key = crypto.derive_sub_key(b"encryption")?;
let mac_key = crypto.derive_sub_key(b"mac")?;

// Get salt for persistence
let salt = crypto.get_salt()?;

// Securely clear sensitive data
crypto.clear();
```

## Security Considerations

1. **Memory Hardening**: Argon2id parameters are automatically adjusted based on device capability
2. **Unique Nonces**: Every encryption operation generates a unique nonce
3. **Secure Clearing**: Sensitive data is zeroized when dropped
4. **Authenticated Encryption**: All encryption includes integrity verification

## Performance

Target performance on typical hardware (iPhone 15 equivalent):
- Key derivation: < 500ms
- Encryption: < 10ms for 1KB data
- Decryption: < 10ms for 1KB data

Run benchmarks: `cargo bench --bench crypto-bench`
