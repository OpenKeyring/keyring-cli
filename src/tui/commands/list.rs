//! TUI List Command Handler
//!
//! Handles the /list command in TUI mode.

use crate::cli::{ConfigManager, onboarding};
use crate::crypto::record::decrypt_payload;
use crate::db::Vault;
use crate::error::Result;
use std::path::PathBuf;

/// Handle the /list command
pub fn handle_list(args: Vec<&str>) -> Result<Vec<String>> {
    let mut output = vec!["📋 Password Records".to_string()];

    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Unlock keystore to decrypt record names
    let crypto = onboarding::unlock_keystore()?;

    let vault = Vault::open(&db_path, "")?;
    let records = vault.list_records()?;

    // Apply filter if provided
    let filter = args.first().map(|s| s.to_lowercase());
    let filtered: Vec<_> = if let Some(filter_str) = filter {
        records
            .into_iter()
            .filter(|r| {
                // Try to decrypt name for filtering
                if let Ok(payload) = decrypt_payload(&crypto, &r.encrypted_data, &r.nonce) {
                    payload.name.to_lowercase().contains(&filter_str)
                } else {
                    false
                }
            })
            .collect()
    } else {
        records.into_iter().collect()
    };

    if filtered.is_empty() {
        output.push("".to_string());
        output.push("No records found.".to_string());
        if args.is_empty() {
            output.push("Use /new to create a record.".to_string());
        } else {
            output.push(format!("No records matching '{}'", args.join(" ")));
        }
    } else {
        output.push("".to_string());
        output.push(format!("Found {} records:", filtered.len()));
        output.push("".to_string());

        for record in filtered {
            // Try to decrypt the record name
            let (name, record_type) = if let Ok(payload) = decrypt_payload(&crypto, &record.encrypted_data, &record.nonce) {
                (payload.name, format!("{:?}", record.record_type).to_lowercase())
            } else {
                (record.id.to_string(), "unknown".to_string())
            };
            output.push(format!("  • {} ({})", name, record_type));
        }
    }

    Ok(output)
}
