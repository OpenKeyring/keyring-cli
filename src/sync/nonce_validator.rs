//! Nonce Validation for Sync Operations
//!
//! This module provides nonce verification to detect potential tampering
//! during sync operations. Each encrypted record has a unique nonce used
//! during AES-256-GCM encryption. If the nonce differs between local and
//! remote versions, it may indicate:
//! - Legitimate re-encryption with updated data
//! - Potential tampering or corruption
//!
//! The validator helps identify these cases and provides recovery strategies.

use crate::db::models::StoredRecord;
use crate::error::KeyringError;
use crate::sync::export::SyncRecord;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::io::{self, Write};

/// Status of nonce validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonceStatus {
    /// Nonce matches - record is consistent
    Valid,
    /// Nonce differs - potential tampering or legitimate update
    Mismatch,
}

impl std::fmt::Display for NonceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NonceStatus::Valid => write!(f, "Nonce is valid"),
            NonceStatus::Mismatch => write!(f, "Nonce mismatch detected"),
        }
    }
}

/// Recovery strategy for nonce mismatches
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// No action needed - nonce is valid
    NoAction,
    /// Ask user to choose between local and remote versions
    AskUser,
    /// Skip this record during sync
    SkipRecord,
    /// Use local version (overwrite remote)
    UseLocal,
    /// Use remote version (overwrite local)
    UseRemote,
}

impl std::fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryStrategy::NoAction => write!(f, "No action needed"),
            RecoveryStrategy::AskUser => write!(f, "User resolution required"),
            RecoveryStrategy::SkipRecord => write!(f, "Skip this record"),
            RecoveryStrategy::UseLocal => write!(f, "Keep local version"),
            RecoveryStrategy::UseRemote => write!(f, "Use remote version"),
        }
    }
}

/// Nonce validator for detecting sync inconsistencies
pub struct NonceValidator;

impl NonceValidator {
    /// Create a new nonce validator
    pub fn new() -> Self {
        Self
    }

    /// Validate nonce between local and remote records
    ///
    /// Returns `Ok(NonceStatus)` indicating whether nonces match,
    /// or `Err(KeyringError)` if validation fails (e.g., corrupted data).
    ///
    /// # Arguments
    /// * `local` - Local stored record
    /// * `remote` - Remote sync record
    ///
    /// # Returns
    /// * `Ok(NonceStatus::Valid)` - Nonces match
    /// * `Ok(NonceStatus::Mismatch)` - Nonces differ
    /// * `Err(KeyringError)` - Invalid nonce encoding or length
    pub fn validate(
        &self,
        local: &StoredRecord,
        remote: &SyncRecord,
    ) -> Result<NonceStatus, KeyringError> {
        // Decode remote nonce from base64
        let remote_nonce_bytes =
            STANDARD
                .decode(&remote.nonce)
                .map_err(|e| KeyringError::Crypto {
                    context: format!("Invalid remote nonce encoding: {}", e),
                })?;

        // Check nonce length (should be 12 bytes for AES-GCM)
        if remote_nonce_bytes.len() != 12 {
            return Err(KeyringError::Crypto {
                context: format!(
                    "Invalid remote nonce length: {} (expected 12)",
                    remote_nonce_bytes.len()
                ),
            });
        }

        // Compare nonces
        if local.nonce == remote_nonce_bytes.as_slice() {
            Ok(NonceStatus::Valid)
        } else {
            Ok(NonceStatus::Mismatch)
        }
    }

    /// Get recommended recovery strategy for a given nonce status
    ///
    /// # Arguments
    /// * `status` - The nonce validation status
    ///
    /// # Returns
    /// The recommended recovery strategy
    pub fn get_recovery_strategy(&self, status: NonceStatus) -> RecoveryStrategy {
        match status {
            NonceStatus::Valid => RecoveryStrategy::NoAction,
            NonceStatus::Mismatch => RecoveryStrategy::AskUser,
        }
    }

    /// Prompt user for resolution of nonce mismatch
    ///
    /// This method displays an interactive prompt to the user asking them
    /// to choose how to resolve a nonce mismatch between local and remote records.
    ///
    /// # Arguments
    /// * `local_nonce` - The local nonce (12 bytes)
    /// * `remote_nonce` - The remote nonce (12 bytes)
    ///
    /// # Returns
    /// * `Ok(RecoveryStrategy)` - User's choice
    /// * `Err(KeyringError)` - User cancelled or input error
    pub fn prompt_user_resolution(
        &self,
        local_nonce: &[u8; 12],
        remote_nonce: &[u8; 12],
    ) -> Result<RecoveryStrategy, KeyringError> {
        #[allow(clippy::print_stdout)]
        {
            println!();
            println!("⚠️  Nonce mismatch detected!");
            println!("Local nonce:  {}", STANDARD.encode(local_nonce));
            println!("Remote nonce: {}", STANDARD.encode(remote_nonce));
            println!();
            println!("This usually means the cloud data belongs to a different vault.");
            println!();
            println!("Possible causes:");
            println!("  • Cloud is from a different vault (Passkey differs)");
            println!("  • Cloud data is corrupted");
            println!("  • Local file was modified");
            println!();
            println!("How to handle?");
            println!("  [1] Use local nonce (overwrite cloud)");
            println!("  [2] Use remote nonce (overwrite local)");
            println!("  [3] Cancel");
        }

        // Flush stdout to ensure the prompt is displayed
        io::stdout()
            .flush()
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        // Read user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        let choice = input.trim();

        Ok(match choice {
            "1" => RecoveryStrategy::UseLocal,
            "2" => RecoveryStrategy::UseRemote,
            "3" => {
                return Err(KeyringError::AuthenticationFailed {
                    reason: "Sync cancelled by user".to_string(),
                })
            }
            _ => {
                // Empty input (non-interactive) or invalid choice defaults to UseLocal
                RecoveryStrategy::UseLocal
            }
        })
    }
}

impl Default for NonceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = NonceValidator::new();
        let _ = validator;
    }

    #[test]
    fn test_validator_default() {
        let validator = NonceValidator::default();
        let _ = validator;
    }
}
