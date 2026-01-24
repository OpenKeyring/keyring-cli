use crate::cli::ConfigManager;
use crate::error::{KeyringError, Result};
use crate::db::Vault;
use std::path::PathBuf;
use std::io::{self, Write};

/// Config command subcommands (matches main.rs)
#[derive(Debug)]
pub enum ConfigCommands {
    Set { key: String, value: String },
    Get { key: String },
    List,
    Reset { force: bool },
}

/// Execute the config command
pub async fn execute(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Set { key, value } => execute_set(key, value).await,
        ConfigCommands::Get { key } => execute_get(key).await,
        ConfigCommands::List => execute_list().await,
        ConfigCommands::Reset { force } => execute_reset(force).await,
    }
}

async fn execute_set(key: String, value: String) -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    // Validate key
    let valid_keys = [
        "sync.path",
        "sync.enabled",
        "sync.auto",
        "clipboard.timeout",
        "clipboard.smart_clear",
        "device_id",
    ];

    if !valid_keys.contains(&key.as_str()) {
        return Err(KeyringError::InvalidInput {
            context: format!("Unknown configuration key: {}. Valid keys: {}", key, valid_keys.join(", ")),
        }.into());
    }

    // Store in metadata table
    vault.set_metadata(&key, &value)?;

    println!("✅ Set {} = {}", key, value);

    Ok(())
}

async fn execute_get(key: String) -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let vault = Vault::open(&db_path, "")?;

    match vault.get_metadata(&key)? {
        Some(value) => println!("{}", value),
        None => println!("(not set)"),
    }

    Ok(())
}

async fn execute_list() -> Result<()> {
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path_str = db_config.path.clone();
    let db_path = PathBuf::from(&db_path_str);
    let vault = Vault::open(&db_path, "")?;

    println!("Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get all metadata
    let all_tags = vault.list_tags()?;
    
    // Get sync config
    let sync_config = config.get_sync_config()?;
    
    // Get clipboard config
    let clipboard_config = config.get_clipboard_config()?;

    // Print sections
    println!("\n[Sync]");
    println!("  sync.enabled = {}", sync_config.enabled);
    println!("  sync.provider = {}", sync_config.provider);
    println!("  sync.remote_path = {}", sync_config.remote_path);
    println!("  sync.auto = {}", sync_config.auto_sync);
    println!("  sync.conflict_resolution = {}", sync_config.conflict_resolution);

    println!("\n[Clipboard]");
    println!("  clipboard.timeout = {} seconds", clipboard_config.timeout_seconds);
    println!("  clipboard.clear_after_copy = {}", clipboard_config.clear_after_copy);
    println!("  clipboard.max_content_length = {}", clipboard_config.max_content_length);

    println!("\n[Database]");
    println!("  database.path = {}", db_path_str);
    println!("  database.encryption_enabled = {}", db_config.encryption_enabled);

    // Print metadata entries
    if !all_tags.is_empty() {
        println!("\n[Metadata]");
        for tag in all_tags {
            if let Some(value) = vault.get_metadata(&tag)? {
                println!("  {} = {}", tag, value);
            }
        }
    }

    Ok(())
}

async fn execute_reset(force: bool) -> Result<()> {
    if !force {
        println!("Are you sure you want to reset all configuration to defaults?");
        print!("Type 'yes' to confirm: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim() != "yes" {
            println!("❌ Reset cancelled");
            return Ok(());
        }
    }

    // TODO: Implement config reset
    // This would reset config.yaml to defaults
    println!("⚠️  Config reset not yet fully implemented");
    println!("✅ Configuration reset requested");

    Ok(())
}
