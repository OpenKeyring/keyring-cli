use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::Result;
use clap::Subcommand;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// List all configuration
    List,
    /// Reset configuration to defaults
    Reset {
        /// Confirm reset
        #[clap(long, short)]
        force: bool,
    },
}

pub async fn execute(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Set { key, value } => execute_set(key, value).await,
        ConfigCommands::Get { key } => execute_get(key).await,
        ConfigCommands::List => execute_list().await,
        ConfigCommands::Reset { force } => execute_reset(force).await,
    }
}

async fn execute_set(key: String, value: String) -> Result<()> {
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

    if !valid_keys.contains(&key.as_str()) {
        return Err(crate::error::Error::ConfigurationError {
            context: format!(
                "Invalid configuration key '{}'. Valid keys are:\n  {}",
                key,
                valid_keys.join("\n  ")
            ),
        });
    }

    println!("⚙️  Setting configuration: {} = {}", key, value);

    // Open vault and persist to metadata
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    vault.set_metadata(&key, &value)?;
    println!("✓ Configuration saved successfully");

    Ok(())
}

async fn execute_get(key: String) -> Result<()> {
    let config = ConfigManager::new()?;

    // Try to get the value from different config sections
    let known_key = match key.as_str() {
        "sync.enabled" => {
            let sync_config = config.get_sync_config()?;
            println!("sync.enabled = {}", sync_config.enabled);
            true
        }
        "sync.provider" => {
            let sync_config = config.get_sync_config()?;
            println!("sync.provider = {}", sync_config.provider);
            true
        }
        "sync.remote_path" => {
            let sync_config = config.get_sync_config()?;
            println!("sync.remote_path = {}", sync_config.remote_path);
            true
        }
        "sync.auto" => {
            let sync_config = config.get_sync_config()?;
            println!("sync.auto = {}", sync_config.auto_sync);
            true
        }
        "sync.conflict_resolution" => {
            let sync_config = config.get_sync_config()?;
            println!(
                "sync.conflict_resolution = {}",
                sync_config.conflict_resolution
            );
            true
        }
        "clipboard.timeout" => {
            let clipboard_config = config.get_clipboard_config()?;
            println!(
                "clipboard.timeout = {} seconds",
                clipboard_config.timeout_seconds
            );
            true
        }
        "database.path" => {
            let db_config = config.get_database_config()?;
            println!("database.path = {}", db_config.path);
            true
        }
        _ => false,
    };

    // If not a known key, check metadata for custom config
    if !known_key {
        let db_config = config.get_database_config()?;
        let db_path = PathBuf::from(db_config.path);
        let vault = Vault::open(&db_path, "")?;

        match vault.get_metadata(&key)? {
            Some(value) => {
                println!("{} = {}", key, value);
            }
            None => {
                println!("Unknown configuration key: {}", key);
            }
        }
    }

    Ok(())
}

async fn execute_list() -> Result<()> {
    let config = ConfigManager::new()?;

    println!("Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get database config
    let db_config = config.get_database_config()?;
    println!("\n[Database]");
    println!("  database.path = {}", db_config.path);
    println!(
        "  database.encryption_enabled = {}",
        db_config.encryption_enabled
    );

    // Get sync config
    let sync_config = config.get_sync_config()?;
    println!("\n[Sync]");
    println!("  sync.enabled = {}", sync_config.enabled);
    println!("  sync.provider = {}", sync_config.provider);
    println!("  sync.remote_path = {}", sync_config.remote_path);
    println!("  sync.auto = {}", sync_config.auto_sync);
    println!(
        "  sync.conflict_resolution = {}",
        sync_config.conflict_resolution
    );

    // Get clipboard config
    let clipboard_config = config.get_clipboard_config()?;
    println!("\n[Clipboard]");
    println!(
        "  clipboard.timeout = {} seconds",
        clipboard_config.timeout_seconds
    );
    println!(
        "  clipboard.clear_after_copy = {}",
        clipboard_config.clear_after_copy
    );
    println!(
        "  clipboard.max_content_length = {}",
        clipboard_config.max_content_length
    );

    Ok(())
}

async fn execute_reset(force: bool) -> Result<()> {
    if !force {
        println!("⚠️  This will reset all custom configuration to defaults.");
        println!("   Custom configuration keys (starting with 'custom.') will be removed.");
        print!("\nContinue? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            println!("Reset cancelled.");
            return Ok(());
        }
    }

    println!("🔄 Configuration reset to defaults");

    // Open vault and clear all custom metadata (keys starting with "custom.")
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    let custom_keys = vault.list_metadata_keys("custom.")?;
    for key in &custom_keys {
        vault.delete_metadata(key)?;
    }

    if !custom_keys.is_empty() {
        println!(
            "   ✓ Cleared {} custom configuration value(s)",
            custom_keys.len()
        );
    } else {
        println!("   No custom configuration to clear");
    }

    Ok(())
}
