//! OpenKeyring Core Library
//!
//! A privacy-first password manager with local-first architecture.

pub mod clipboard;
pub mod crypto;
pub mod db;
pub mod device;
pub mod error;
pub mod mcp;
pub mod onboarding;
pub mod sync;

pub use error::Result;
