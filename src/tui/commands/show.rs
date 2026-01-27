//! TUI Show Command Handler
//!
//! Handles the /show command in TUI mode.

use crate::cli::{onboarding, ConfigManager};
use crate::crypto::record::decrypt_payload;
use crate::db::Vault;
use crate::error::Result;
use std::path::PathBuf;

/// Handle the /show command
#[allow(dead_code)]
pub fn handle_show(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return Ok(vec![
            "❌ Error: Record name required".to_string(),
            "Usage: /show <name>".to_string(),
        ]);
    }

    let name = args[0];

    // Ensure vault is initialized
    onboarding::ensure_initialized()?;

    // Unlock keystore
    let crypto = onboarding::unlock_keystore()?;

    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Open vault
    let vault = Vault::open(&db_path, "")?;

    // Get all records and search by name (since names are encrypted)
    let records = vault.list_records()?;

    // Decrypt records to find the matching one
    let mut matched_record = None;
    for record in records {
        if let Ok(payload) = decrypt_payload(&crypto, &record.encrypted_data, &record.nonce) {
            if payload.name == name {
                matched_record = Some((record, payload));
                break;
            }
        }
    }

    let (_record, decrypted_payload) = match matched_record {
        Some(r) => r,
        None => {
            return Ok(vec![
                format!("❌ Record '{}' not found", name),
                "Use /list to see all records.".to_string(),
            ]);
        }
    };

    // Format output for TUI display
    let mut output = vec![
        format!("🔑 Record: {}", decrypted_payload.name),
        "".to_string(),
    ];

    // Username
    if let Some(ref username) = decrypted_payload.username {
        output.push(format!("👤 Username: {}", username));
    }

    // Password (will be shown in popup in TUI)
    output.push("🔐 Password: *** (shown in popup)".to_string());

    // URL
    if let Some(ref url) = decrypted_payload.url {
        output.push(format!("🔗 URL: {}", url));
    }

    // Notes
    if let Some(ref notes) = decrypted_payload.notes {
        if !notes.is_empty() {
            output.push(format!("📝 Notes: {}", notes));
        }
    }

    // Tags
    if !decrypted_payload.tags.is_empty() {
        output.push(format!("🏷️  Tags: {}", decrypted_payload.tags.join(", ")));
    }

    output.push("".to_string());
    output.push("(Password copied to clipboard - auto-clears in 30s)".to_string());

    Ok(output)
}
