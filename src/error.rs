//! Error types for OpenKeyring
//!
//! This module defines all error types used throughout the application.

use thiserror::Error;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

/// Keyring error alias for backward compatibility
pub type KeyringError = Error;

/// OpenKeyring error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("Crypto error: {context}")]
    Crypto { context: String },

    #[error("Database error: {context}")]
    Database { context: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Invalid input: {context}")]
    InvalidInput { context: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Internal error: {context}")]
    Internal { context: String },

    #[error("Clipboard error: {context}")]
    Clipboard { context: String },

    #[error("Sync error: {context}")]
    Sync { context: String },

    #[error("MCP error: {context}")]
    Mcp { context: String },

    #[error("Unsupported platform")]
    UnsupportedPlatform,

    #[error("Clipboard not supported")]
    ClipboardNotSupported,

    #[error("Content too long: {max_length}")]
    ContentTooLong { max_length: usize },

    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    #[error("Tool already exists: {tool_name}")]
    ToolExists { tool_name: String },

    #[error("Configuration error: {context}")]
    ConfigurationError { context: String },

    #[error("Tool timeout: {context}")]
    ToolTimeout { context: String },

    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },

    #[error("Record not found: {name}")]
    RecordNotFound { name: String },

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    IoError(String),
}

// Convert from uuid::Error for compatibility
impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error::Internal {
            context: err.to_string(),
        }
    }
}

// Convert from anyhow::Error for compatibility
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal {
            context: err.to_string(),
        }
    }
}
