//! Error types for OpenKeyring.

use thiserror::Error;

/// Type alias for Result with OpenKeyringError.
pub type Result<T> = std::result::Result<T, OpenKeyringError>;

/// Main error type for OpenKeyring.
#[derive(Error, Debug)]
pub enum OpenKeyringError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
}
