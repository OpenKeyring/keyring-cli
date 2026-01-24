//! OpenKeyring Core Library
//!
//! A privacy-first password manager with local-first architecture.

pub mod crypto;
pub mod db;
pub mod clipboard;
pub mod sync;
pub mod mcp;
pub mod error;

pub use error::Result;
