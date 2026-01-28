//! TUI Delete Command Handler
//!
//! Handles the /delete command in TUI mode with confirmation dialog.

use crate::cli::{onboarding, ConfigManager};
use crate::crypto::record::decrypt_payload;
use crate::db::Vault;
use crate::error::Result;

/// Handle the /delete command with interactive confirmation
pub fn handle_delete(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return Ok(vec![
            "❌ Error: Record name required".to_string(),
            "Usage: /delete <name>".to_string(),
        ]);
    }

    let name = args[0];

    // Try to initialize vault and crypto, but handle errors gracefully
    let display_name = match try_get_record_display_name(name) {
        Some(display_name) => display_name,
        None => {
            // If vault is not initialized or record not found, use the provided name
            // (don't reveal whether a record exists for security)
            name.to_string()
        }
    };

    // Return confirmation prompt (TUI app will handle user input)
    let mut output = vec![
        "⚠️  Delete Confirmation".to_string(),
        "".to_string(),
        format!("Are you sure you want to delete '{}'?", display_name),
        "".to_string(),
        "This action cannot be undone.".to_string(),
        "".to_string(),
        "Type 'yes' to confirm, or anything else to cancel:".to_string(),
    ];

    // In a real TUI with state, we'd handle the confirmation here
    // For now, return the prompt and the caller handles confirmation
    output.extend(vec![
        "".to_string(),
        "(TUI: Implement confirmation dialog - requires state management)".to_string(),
    ]);

    Ok(output)
}

/// Try to get the display name for a record, returning None if not found or error
fn try_get_record_display_name(name: &str) -> Option<String> {
    // Try to initialize vault and crypto, return None on any error
    let _crypto = match (|| {
        onboarding::ensure_initialized()?;
        onboarding::unlock_keystore()
    })() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let config = match ConfigManager::new() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let db_config = match config.get_database_config() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let db_path = std::path::PathBuf::from(db_config.path);

    // Find record by name
    let vault = match Vault::open(&db_path, "") {
        Ok(v) => v,
        Err(_) => return None,
    };

    let record = match vault.find_record_by_name(name) {
        Ok(Some(r)) => r,
        _ => return None,
    };

    // Decrypt to show name in confirmation
    match decrypt_payload(&_crypto, &record.encrypted_data, &record.nonce) {
        Ok(payload) => Some(payload.name),
        Err(_) => Some(name.to_string()),
    }
}

/// Actually delete the record (called after confirmation)
pub fn execute_delete(name: &str) -> Result<Vec<String>> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);

    let mut vault = Vault::open(&db_path, "")?;

    // Find and delete
    let record = match vault.find_record_by_name(name)? {
        Some(r) => r,
        None => {
            return Ok(vec![format!("❌ Record '{}' not found", name)]);
        }
    };

    vault.delete_record(&record.id.to_string())?;

    Ok(vec![
        format!("✅ Record '{}' deleted successfully", name),
        "".to_string(),
    ])
}
