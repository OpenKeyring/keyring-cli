//! TUI Search Command Handler
//!
//! Handles the /search command in TUI mode with fuzzy matching.

use crate::cli::{onboarding, ConfigManager};
use crate::crypto::record::decrypt_payload;
use crate::db::Vault;
use crate::error::Result;

/// Handle the /search command with fuzzy matching
#[allow(dead_code)]
pub fn handle_search(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return Ok(vec![
            "❌ Error: Search query required".to_string(),
            "Usage: /search <query>".to_string(),
        ]);
    }

    let query = args.join(" ").to_lowercase();

    // Initialize
    onboarding::ensure_initialized()?;
    let crypto = onboarding::unlock_keystore()?;
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);

    let vault = Vault::open(&db_path, "")?;
    let records = vault.list_records()?;

    // Search with fuzzy matching
    let mut results = vec![];
    for record in records {
        if let Ok(payload) = decrypt_payload(&crypto, &record.encrypted_data, &record.nonce) {
            // Check name match
            if payload.name.to_lowercase().contains(&query) {
                results.push((record, payload, "name".to_string()));
                continue;
            }
            // Check username match
            if let Some(ref username) = payload.username {
                if username.to_lowercase().contains(&query) {
                    results.push((record, payload, "username".to_string()));
                    continue;
                }
            }
            // Check URL match
            if let Some(ref url) = payload.url {
                if url.to_lowercase().contains(&query) {
                    results.push((record, payload, "url".to_string()));
                    continue;
                }
            }
            // Check tags match
            let matched_tag: Option<String> = payload.tags.iter()
                .find(|tag| tag.to_lowercase().contains(&query))
                .map(|tag| tag.clone());
            if let Some(tag) = matched_tag {
                results.push((record, payload, format!("tag: {}", tag)));
                continue;
            }
        }
    }

    // Format results
    if results.is_empty() {
        return Ok(vec![
            format!("🔍 No results found for '{}'", query),
            "".to_string(),
            "Tips:".to_string(),
            "  - Try a shorter query".to_string(),
            "  - Check spelling".to_string(),
            "  - Use /list to see all records".to_string(),
        ]);
    }

    let mut output = vec![
        format!("🔍 Found {} results for '{}':", results.len(), query),
        "".to_string(),
    ];

    for (_record, payload, matched_by) in results {
        output.push(format!("• {} (matched by: {})", payload.name, matched_by));
        if let Some(ref username) = payload.username {
            output.push(format!("  Username: {}", username));
        }
        if let Some(ref url) = payload.url {
            output.push(format!("  URL: {}", url));
        }
        output.push("".to_string());
    }

    Ok(output)
}
