//! TUI Widgets
//!
//! Reusable UI components for the TUI interface.

// Widgets are part of the TUI API but may not all be used yet
#![allow(dead_code)]

mod password;
mod mnemonic;
mod input;

pub use password::PasswordPopup;
pub use mnemonic::MnemonicDisplay;
pub use input::CommandInput;
