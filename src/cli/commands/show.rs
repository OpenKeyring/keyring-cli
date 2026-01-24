use crate::cli::{onboarding, ConfigManager};
use crate::crypto::record::decrypt_payload;
use crate::db::Vault;
use crate::error::{KeyringError, Result};
use std::path::PathBuf;

/// Execute the show command
pub async fn execute(
    name: String,
    password: bool,
    copy: bool,
    timeout: Option<u64>,
    field: Option<String>,
    history: bool,
) -> Result<()> {
    // Ensure vault is initialized
    onboarding::ensure_initialized()?;

    // Unlock keystore
    let crypto = onboarding::unlock_keystore()?;

    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Open vault
    let vault = Vault::open(&db_path, "")?;

    // Search for record by name (using search_records)
    // We need to decrypt records to find the matching name
    let records = vault.search_records(&name)?;
    
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
    
    let (_record, decrypted_payload) = matched_record
        .ok_or_else(|| KeyringError::NotFound {
            resource: format!("Record with name '{}'", name),
        })?;

    // Handle copy to clipboard
    if copy {
        use crate::clipboard::{create_platform_clipboard, ClipboardConfig, ClipboardService};
        let clipboard_manager = create_platform_clipboard()?;
        let clipboard_config = ClipboardConfig::default();
        let mut clipboard = ClipboardService::new(clipboard_manager, clipboard_config);
        clipboard.copy_password(&decrypted_payload.password)?;
        
        let timeout_secs = timeout.unwrap_or(30);
        println!("📋 Password copied to clipboard (auto-clears in {} seconds)", timeout_secs);
        
        return Ok(());
    }

    // Show specific field
    if let Some(field_name) = field {
        match field_name.as_str() {
            "name" => println!("{}", decrypted_payload.name),
            "username" => println!("{}", decrypted_payload.username.as_deref().unwrap_or("")),
            "password" => {
                if password {
                    println!("{}", decrypted_payload.password);
                } else {
                    println!("••••••••••••");
                }
            }
            "url" => println!("{}", decrypted_payload.url.as_deref().unwrap_or("")),
            "notes" => println!("{}", decrypted_payload.notes.as_deref().unwrap_or("")),
            _ => return Err(KeyringError::InvalidInput {
                context: format!("Unknown field: {}", field_name),
            }),
        }
        return Ok(());
    }

    // Show history (not yet implemented)
    if history {
        println!("⚠️  History feature not yet implemented");
        return Ok(());
    }

    // Show full record (decrypted)
    println!("Name: {}", decrypted_payload.name);
    if let Some(ref username) = decrypted_payload.username {
        println!("Username: {}", username);
    }
    if password {
        println!("Password: {}", decrypted_payload.password);
    } else {
        println!("Password: ••••••••••••");
    }
    if let Some(ref url) = decrypted_payload.url {
        println!("URL: {}", url);
    }
    if let Some(ref notes) = decrypted_payload.notes {
        println!("Notes: {}", notes);
    }
    if !decrypted_payload.tags.is_empty() {
        println!("Tags: {}", decrypted_payload.tags.join(", "));
    }

    Ok(())
}

// Legacy function for backward compatibility
#[derive(clap::Parser, Debug)]
pub struct ShowArgs {
    pub name: String,
    #[clap(short, long)]
    pub show_password: bool,
    #[clap(short, long)]
    pub copy_to_clipboard: bool,
    #[clap(long)]
    pub sync: bool,
}

pub async fn show_record(args: ShowArgs) -> Result<()> {
    execute(
        args.name,
        args.show_password,
        args.copy_to_clipboard,
        None,
        None,
        false,
    )
    .await
}
