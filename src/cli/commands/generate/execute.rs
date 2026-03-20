//! Execute password generation command
//!
//! Main command execution and helper functions.

use super::args::{get_password_description, NewArgs, PasswordType};
use super::generators;
use crate::cli::ConfigManager;
use crate::clipboard::{create_platform_clipboard, ClipboardConfig, ClipboardService};
use crate::crypto::{
    keystore::KeyStore,
    record::{encrypt_payload, RecordPayload},
    CryptoManager,
};
use crate::db::models::RecordType;
use crate::db::vault::Vault;
use crate::error::{KeyringError, Result};
use crate::onboarding::is_initialized;
use std::io::Write;
use std::path::PathBuf;

/// Execute the generate command
pub async fn execute(args: NewArgs) -> Result<()> {
    // Validate arguments
    args.validate()?;

    // Initialize configuration
    let config = ConfigManager::new()?;

    // Initialize keystore and crypto manager with DEK
    let master_password = config.get_master_password()?;
    let keystore_path = config.get_keystore_path();
    let keystore = if is_initialized(&keystore_path) {
        KeyStore::unlock(&keystore_path, &master_password)?
    } else {
        let keystore = KeyStore::initialize(&keystore_path, &master_password)?;
        if let Some(recovery_key) = &keystore.recovery_key {
            println!("🔑 Recovery Key (save securely): {}", recovery_key);
        }
        keystore
    };
    let mut crypto = CryptoManager::new();
    let dek = keystore.get_dek();
    let dek_array: [u8; 32] = dek.try_into().map_err(|_| {
        KeyringError::Crypto {
            context: "Invalid DEK length: expected 32 bytes".to_string(),
        }
    })?;
    crypto.initialize_with_key(dek_array);

    // Generate password based on type
    let password_type = args.get_password_type()?;
    let password = match password_type {
        PasswordType::Random => generators::generate_random(args.length, args.numbers, args.symbols)?,
        PasswordType::Memorable => generators::generate_memorable(args.words)?,
        PasswordType::Pin => generators::generate_pin(args.length)?,
    };

    let payload = RecordPayload {
        name: args.name.clone(),
        username: args.username.clone(),
        password: password.clone(),
        url: args.url.clone(),
        notes: args
            .notes
            .clone()
            .or(Some(get_password_description(password_type))),
        tags: args.tags.clone(),
    };
    let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;

    // Create record
    let record = crate::db::models::StoredRecord {
        id: uuid::Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags: args.tags.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1, // New records start at version 1
    };

    // Get database path
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save to database
    let mut vault = Vault::open(&db_path, &master_password)?;
    vault.add_record(&record)?;

    // Copy to clipboard if requested
    // Use --no-copy to display password in terminal (useful for testing/automation)
    if args.copy {
        copy_to_clipboard(&password)?;
        print_success_message(&args.name, password_type, true);
    } else {
        print_success_message(&args.name, password_type, false);
        // Display password when --no-copy is used
        println!("   Password: {}", password);
    }

    // Handle sync if requested
    if args.sync {
        println!("🔄 Sync to cloud requested (not yet implemented)");
    }

    Ok(())
}

/// Prompt user for master password
#[allow(dead_code)]
pub fn prompt_for_master_password() -> Result<String> {
    use rpassword::read_password;

    print!("🔐 Enter master password: ");
    let _ = std::io::stdout().flush();
    let password = read_password()
        .map_err(|e| KeyringError::IoError(format!("Failed to read password: {}", e)))?;

    if password.is_empty() {
        return Err(KeyringError::AuthenticationFailed {
            reason: "Master password cannot be empty".to_string(),
        });
    }

    Ok(password)
}

/// Copy password to clipboard securely
fn copy_to_clipboard(password: &str) -> Result<()> {
    let clipboard_manager = create_platform_clipboard()?;
    let mut clipboard = ClipboardService::new(clipboard_manager, ClipboardConfig::default());

    clipboard.copy_password(password)?;
    Ok(())
}

/// Print success message with password details
fn print_success_message(name: &str, password_type: PasswordType, copied: bool) {
    println!("✅ Password generated successfully!");
    println!("   Name: {}", name);
    println!("   Type: {}", format!("{:?}", password_type).to_lowercase());

    // Clipboard notice (only when copied)
    if copied {
        println!("   📋 Copied to clipboard (auto-clears in 30s)");
    }
}

// Re-export for backward compatibility
pub use execute as generate_password;
