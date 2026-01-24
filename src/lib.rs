//! OpenKeyring Core Library
//!
//! A privacy-first password manager with local-first architecture.

pub mod clipboard;
pub mod crypto;
pub mod db;
pub mod error;
pub mod mcp;
pub mod sync;

pub use clipboard::{ClipboardManager, ClipboardService};
pub use clipboard::manager::ClipboardConfig as ClipboardManagerConfig;
pub use crypto::{CryptoManager, KeyDerivation, EncryptionMethod};
pub use db::{DatabaseManager, RecordManager, TagManager};
pub use error::{KeyringError, Result};
pub use mcp::{McpServer, ServerConfig, AuthManager};
pub use sync::{SyncExporter, SyncImporter, SyncConfig, SyncStatus};

// Public exports for CLI
pub mod cli;
pub use cli::*;
