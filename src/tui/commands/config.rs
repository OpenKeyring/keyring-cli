//! TUI Config Command Handler
//!
//! Handles the /config command in TUI mode.

use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::Result;
use std::path::PathBuf;

/// Handle the /config command
#[allow(dead_code)]
pub fn handle_config(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return handle_config_list();
    }

    let subcommand = args[0];
    let sub_args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };

    match subcommand {
        "list" | "ls" => handle_config_list(),
        "set" => handle_config_set(sub_args),
        "get" => handle_config_get(sub_args),
        "reset" => handle_config_reset(sub_args),
        _ => Ok(vec![
            "❌ Unknown config subcommand".to_string(),
            "".to_string(),
            "Usage:".to_string(),
            "  /config list              - List all configuration".to_string(),
            "  /config get <key>         - Get a configuration value".to_string(),
            "  /config set <key> <value> - Set a configuration value".to_string(),
            "  /config reset             - Reset configuration to defaults".to_string(),
            "".to_string(),
        ]),
    }
}

/// List all configuration
fn handle_config_list() -> Result<Vec<String>> {
    let config = ConfigManager::new()?;

    let mut output = vec![
        "⚙️  Configuration".to_string(),
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
        "".to_string(),
    ];

    // Get database config
    let db_config = config.get_database_config()?;
    output.push("[Database]".to_string());
    output.push(format!("  database.path = {}", db_config.path));
    output.push(format!(
        "  database.encryption_enabled = {}",
        db_config.encryption_enabled
    ));
    output.push("".to_string());

    // Get sync config
    let sync_config = config.get_sync_config()?;
    output.push("[Sync]".to_string());
    output.push(format!("  sync.enabled = {}", sync_config.enabled));
    output.push(format!("  sync.provider = {}", sync_config.provider));
    output.push(format!("  sync.remote_path = {}", sync_config.remote_path));
    output.push(format!("  sync.auto = {}", sync_config.auto_sync));
    output.push(format!(
        "  sync.conflict_resolution = {}",
        sync_config.conflict_resolution
    ));
    output.push("".to_string());

    // Get clipboard config
    let clipboard_config = config.get_clipboard_config()?;
    output.push("[Clipboard]".to_string());
    output.push(format!(
        "  clipboard.timeout = {} seconds",
        clipboard_config.timeout_seconds
    ));
    output.push(format!(
        "  clipboard.clear_after_copy = {}",
        clipboard_config.clear_after_copy
    ));
    output.push(format!(
        "  clipboard.max_content_length = {}",
        clipboard_config.max_content_length
    ));

    Ok(output)
}

/// Get a configuration value
fn handle_config_get(args: Vec<&str>) -> Result<Vec<String>> {
    if args.is_empty() {
        return Ok(vec![
            "❌ Error: Configuration key required".to_string(),
            "Usage: /config get <key>".to_string(),
        ]);
    }

    let key = args[0];
    let config = ConfigManager::new()?;

    // Try to get the value from different config sections
    let known_key = match key {
        "sync.enabled" => {
            let sync_config = config.get_sync_config()?;
            Some(format!("sync.enabled = {}", sync_config.enabled))
        }
        "sync.provider" => {
            let sync_config = config.get_sync_config()?;
            Some(format!("sync.provider = {}", sync_config.provider))
        }
        "sync.remote_path" => {
            let sync_config = config.get_sync_config()?;
            Some(format!("sync.remote_path = {}", sync_config.remote_path))
        }
        "sync.auto" => {
            let sync_config = config.get_sync_config()?;
            Some(format!("sync.auto = {}", sync_config.auto_sync))
        }
        "sync.conflict_resolution" => {
            let sync_config = config.get_sync_config()?;
            Some(format!(
                "sync.conflict_resolution = {}",
                sync_config.conflict_resolution
            ))
        }
        "clipboard.timeout" => {
            let clipboard_config = config.get_clipboard_config()?;
            Some(format!(
                "clipboard.timeout = {} seconds",
                clipboard_config.timeout_seconds
            ))
        }
        "database.path" => {
            let db_config = config.get_database_config()?;
            Some(format!("database.path = {}", db_config.path))
        }
        _ => None,
    };

    // If not a known key, check metadata for custom config
    if let Some(value) = known_key {
        Ok(vec![value])
    } else {
        let db_config = config.get_database_config()?;
        let db_path = PathBuf::from(db_config.path);
        let vault = Vault::open(&db_path, "")?;

        match vault.get_metadata(key)? {
            Some(value) => Ok(vec![format!("{} = {}", key, value)]),
            None => Ok(vec![
                format!("❌ Unknown configuration key: '{}'", key),
                "".to_string(),
                "Valid keys:".to_string(),
                "  sync.enabled, sync.provider, sync.remote_path".to_string(),
                "  sync.auto, sync.conflict_resolution".to_string(),
                "  clipboard.timeout, database.path".to_string(),
            ]),
        }
    }
}

/// Set a configuration value
fn handle_config_set(args: Vec<&str>) -> Result<Vec<String>> {
    if args.len() < 2 {
        return Ok(vec![
            "❌ Error: Key and value required".to_string(),
            "Usage: /config set <key> <value>".to_string(),
            "".to_string(),
            "Valid keys:".to_string(),
            "  sync.path, sync.enabled, sync.auto".to_string(),
            "  sync.provider, sync.remote_path, sync.conflict_resolution".to_string(),
            "  clipboard.timeout, clipboard.smart_clear".to_string(),
            "  clipboard.clear_after_copy, clipboard.max_content_length".to_string(),
            "  device_id".to_string(),
        ]);
    }

    let key = args[0];
    let value = args[1..].join(" ");

    // Validate configuration key
    let valid_keys = [
        "sync.path",
        "sync.enabled",
        "sync.auto",
        "sync.provider",
        "sync.remote_path",
        "sync.conflict_resolution",
        "clipboard.timeout",
        "clipboard.smart_clear",
        "clipboard.clear_after_copy",
        "clipboard.max_content_length",
        "device_id",
    ];

    if !valid_keys.contains(&key) {
        return Ok(vec![
            format!("❌ Invalid configuration key '{}'", key),
            "".to_string(),
            "Valid keys:".to_string(),
            format!("  {}", valid_keys.join("\n  ")),
        ]);
    }

    // Open vault and persist to metadata
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    vault.set_metadata(key, &value)?;

    Ok(vec![
        format!("⚙️  Set: {} = {}", key, value),
        "✓ Configuration saved successfully".to_string(),
    ])
}

/// Reset configuration to defaults
fn handle_config_reset(args: Vec<&str>) -> Result<Vec<String>> {
    let force = args.iter().any(|&a| a == "--force" || a == "-f");

    if !force {
        return Ok(vec![
            "⚠️  This will reset all custom configuration to defaults.".to_string(),
            "   Custom configuration keys (starting with 'custom.') will be removed.".to_string(),
            "".to_string(),
            "To confirm, use:".to_string(),
            "  /config reset --force".to_string(),
        ]);
    }

    // Open vault and clear all custom metadata (keys starting with "custom.")
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    let custom_keys = vault.list_metadata_keys("custom.")?;
    for key in &custom_keys {
        vault.delete_metadata(key)?;
    }

    if custom_keys.is_empty() {
        Ok(vec![
            "🔄 Configuration reset to defaults".to_string(),
            "   No custom configuration to clear".to_string(),
        ])
    } else {
        Ok(vec![
            "🔄 Configuration reset to defaults".to_string(),
            format!("   ✓ Cleared {} custom configuration value(s)", custom_keys.len()),
        ])
    }
}
