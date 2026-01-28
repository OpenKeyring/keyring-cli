//! TUI Update Command Handler
//!
//! Handles the /update command in TUI mode with interactive wizard.

use crate::cli::{onboarding, ConfigManager};
use crate::crypto::record::{decrypt_payload, encrypt_payload};
use crate::db::Vault;
use crate::error::Result;

/// Handle the /update command with interactive wizard
#[allow(dead_code)]
pub fn handle_update(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return Ok(vec![
            "❌ Error: Record name required".to_string(),
            "Usage: /update <name>".to_string(),
        ]);
    }

    let name = args[0];

    // Try to get record info for display, fall back to provided name if not available
    let display_info = try_get_record_info(name);

    // Show current values and prompt for updates
    let mut output = vec![
        "✏️  Update Record".to_string(),
        "".to_string(),
        format!("Name: {}", display_info.as_ref().map(|i| i.name.as_str()).unwrap_or(name)),
    ];

    if let Some(ref info) = display_info {
        if let Some(ref username) = info.username {
            output.push(format!("Username: {}", username));
        }
        if let Some(ref url) = info.url {
            output.push(format!("URL: {}", url));
        }
        if let Some(ref notes) = info.notes {
            output.push(format!("Notes: {}", notes));
        }
        if !info.tags.is_empty() {
            output.push(format!("Tags: {}", info.tags.join(", ")));
        }
    }

    output.extend(vec![
        "".to_string(),
        "Enter new values (press Enter to keep current):".to_string(),
        "".to_string(),
        "(TUI: Implement interactive input for each field)".to_string(),
        "".to_string(),
        "Available fields:".to_string(),
        "  - password: Generate new password".to_string(),
        "  - username: <new username>".to_string(),
        "  - url: <new url>".to_string(),
        "  - notes: <new notes>".to_string(),
        "  - tags: <tag1,tag2>".to_string(),
    ]);

    Ok(output)
}

/// Information about a record for display
struct RecordInfo {
    name: String,
    username: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    tags: Vec<String>,
}

/// Try to get the record info, returning None if not found or error
fn try_get_record_info(name: &str) -> Option<RecordInfo> {
    // Try to initialize vault and crypto, return None on any error
    let crypto = match (|| {
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

    // Decrypt to get record info
    match decrypt_payload(&crypto, &record.encrypted_data, &record.nonce) {
        Ok(payload) => Some(RecordInfo {
            name: payload.name,
            username: payload.username,
            url: payload.url,
            notes: payload.notes,
            tags: payload.tags,
        }),
        Err(_) => None,
    }
}

/// Update a specific field
#[allow(dead_code)]
pub fn update_field(name: &str, field: &str, value: &str) -> Result<Vec<String>> {
    let crypto = onboarding::unlock_keystore()?;
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);

    let mut vault = Vault::open(&db_path, "")?;
    let record = match vault.find_record_by_name(name)? {
        Some(r) => r,
        None => {
            return Ok(vec![format!("❌ Record '{}' not found", name)]);
        }
    };

    // Decrypt and parse payload
    let mut payload = decrypt_payload(&crypto, &record.encrypted_data, &record.nonce)?;

    // Update the specified field
    match field {
        "username" => {
            payload.username = if value.is_empty() { None } else { Some(value.to_string()) };
        }
        "url" => {
            payload.url = if value.is_empty() { None } else { Some(value.to_string()) };
        }
        "notes" => {
            payload.notes = if value.is_empty() { None } else { Some(value.to_string()) };
        }
        "tags" => {
            let tags: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
            payload.tags = tags;
        }
        _ => {
            return Ok(vec![format!("❌ Unknown field: {}", field)]);
        }
    }

    let mut record = record;
    record.updated_at = chrono::Utc::now();
    record.tags = payload.tags.clone();

    // Encrypt and save
    let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;
    record.encrypted_data = encrypted_data;
    record.nonce = nonce;
    vault.update_record(&record)?;

    Ok(vec![
        format!("✅ Updated {} for '{}'", field, name),
        "".to_string(),
    ])
}

/// Generate new password for record
#[allow(dead_code)]
pub fn update_password(name: &str, new_password: &str) -> Result<Vec<String>> {
    let crypto = onboarding::unlock_keystore()?;
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);

    let mut vault = Vault::open(&db_path, "")?;
    let record = match vault.find_record_by_name(name)? {
        Some(r) => r,
        None => {
            return Ok(vec![format!("❌ Record '{}' not found", name)]);
        }
    };

    let mut payload = decrypt_payload(&crypto, &record.encrypted_data, &record.nonce)?;
    payload.password = new_password.to_string();

    let mut record = record;
    record.updated_at = chrono::Utc::now();

    let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;
    record.encrypted_data = encrypted_data;
    record.nonce = nonce;
    vault.update_record(&record)?;

    Ok(vec![
        format!("✅ Password updated for '{}'", name),
        "".to_string(),
    ])
}
