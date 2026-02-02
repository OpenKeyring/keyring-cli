//! TUI New Command Handler
//!
//! Handles the /new command in TUI mode with interactive wizard.

use crate::cli::commands::generate::{
    generate_memorable, generate_pin, generate_random, PasswordType,
};
use crate::cli::ConfigManager;
use crate::crypto::record::{encrypt_payload, RecordPayload};
use crate::crypto::{keystore::KeyStore, CryptoManager};
use crate::db::models::{RecordType, StoredRecord};
use crate::db::Vault;
use crate::error::Result;

/// Handle the /new command with interactive wizard
pub fn handle_new() -> Result<Vec<String>> {
    Ok(vec![
        "📝 Create New Password Record".to_string(),
        "".to_string(),
        "The /new command creates a new password record.".to_string(),
        "".to_string(),
        "In CLI mode, use: ok new --name <record-name>".to_string(),
        "".to_string(),
        "Examples:".to_string(),
        "  ok new --name github --username user@example.com".to_string(),
        "  ok new --name email -p --memorable".to_string(),
        "  ok new --name bank --pin --length 6".to_string(),
        "".to_string(),
        "Options:".to_string(),
        "  --name <name>      Record name (required)".to_string(),
        "  --username <user>  Username".to_string(),
        "  --url <url>        Website URL".to_string(),
        "  --notes <notes>    Notes".to_string(),
        "  --tags <tags>      Comma-separated tags".to_string(),
        "  --length <n>       Password length (default: 16)".to_string(),
        "  --memorable        Generate word-based password".to_string(),
        "  --pin              Generate PIN code".to_string(),
        "  --copy             Copy to clipboard after generation".to_string(),
        "".to_string(),
        "For full help: ok new --help".to_string(),
    ])
}

/// Create a new record with generated password
pub fn create_record(
    name: &str,
    password_type: PasswordType,
    password_length: usize,
    username: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    tags: Vec<String>,
) -> Result<Vec<String>> {
    let config = ConfigManager::new()?;
    let master_password = config.get_master_password()?;

    // Initialize crypto
    let keystore_path = config.get_keystore_path();
    let keystore = KeyStore::unlock(&keystore_path, &master_password)?;
    let mut crypto = CryptoManager::new();
    let dek_array: [u8; 32] = keystore.get_dek().try_into().expect("DEK must be 32 bytes");
    crypto.initialize_with_key(dek_array);

    // Generate password
    let password = match password_type {
        PasswordType::Random => generate_random(password_length, true, true)?,
        PasswordType::Memorable => generate_memorable(4)?,
        PasswordType::Pin => generate_pin(password_length)?,
    };

    // Create payload
    let payload = RecordPayload {
        name: name.to_string(),
        username,
        password: password.clone(),
        url,
        notes,
        tags: tags.clone(),
    };

    let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;

    // Create record
    let record = StoredRecord {
        id: uuid::Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1, // New records start at version 1
    };

    // Save
    let db_config = config.get_database_config()?;
    let db_path = std::path::PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, &master_password)?;
    vault.add_record(&record)?;

    Ok(vec![
        "✅ Record created successfully!".to_string(),
        "".to_string(),
        format!("Name: {}", name),
        format!("Password: {}", password),
        format!("Type: {:?}", password_type),
        "".to_string(),
    ])
}
