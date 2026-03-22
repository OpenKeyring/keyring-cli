//! Password generation command (accessible via 'new' subcommand)
//!
//! This module provides password generation functionality with three types:
//! - Random: High-entropy random passwords with special characters
//! - Memorable: Word-based passphrases (e.g., "Correct-Horse-Battery-Staple")
//! - PIN: Numeric PIN codes

pub mod args;
mod execute;
mod generators;
#[cfg(test)]
mod tests;

pub use args::{get_password_description, NewArgs, PasswordType};
pub use execute::{execute, generate_password, prompt_for_master_password};
pub use generators::{generate_memorable, generate_pin, generate_random};
