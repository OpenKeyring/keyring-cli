//! OpenKeyring Core Library
//!
//! A privacy-first password manager with local-first architecture.

pub mod cli;
pub mod cloud;
pub mod clipboard;
pub mod config;
pub mod crypto;
pub mod db;
pub mod device;
pub mod error;
pub mod health;
pub mod mcp;
pub mod onboarding;
pub mod sync;
pub mod tui;
pub mod types;

pub use error::Result;
