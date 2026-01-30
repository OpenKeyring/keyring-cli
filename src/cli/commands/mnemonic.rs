use crate::cli::ConfigManager;
use crate::crypto::bip39;
use crate::crypto::{
    keystore::KeyStore,
    record::{encrypt_payload, RecordPayload},
    CryptoManager,
};
use crate::db::models::{RecordType, StoredRecord};
use crate::db::vault::Vault;
use crate::error::Result;
use crate::onboarding::is_initialized;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct MnemonicArgs {
    #[clap(long, short)]
    pub generate: Option<u8>,
    #[clap(long, short)]
    pub validate: Option<String>,
    #[clap(long, short)]
    pub name: Option<String>,
}

pub async fn handle_mnemonic(args: MnemonicArgs) -> Result<()> {
    if let Some(word_count) = args.generate {
        generate_mnemonic(word_count, args.name).await?;
    } else if let Some(words) = args.validate {
        validate_mnemonic(&words).await?;
    } else {
        println!("Please specify either --generate or --validate");
    }

    Ok(())
}

async fn generate_mnemonic(word_count: u8, name: Option<String>) -> Result<()> {
    let mnemonic = bip39::generate_mnemonic(word_count as usize)?;

    if let Some(name) = name {
        // Create record payload
        let payload = RecordPayload {
            name: name.clone(),
            username: None,
            password: mnemonic.clone(),
            url: None,
            notes: Some(format!("{}-word BIP39 mnemonic phrase for cryptocurrency wallet recovery", word_count)),
            tags: vec!["crypto".to_string(), "wallet".to_string(), "mnemonic".to_string()],
        };

        // Get config
        let config_manager = ConfigManager::new()?;

        // Initialize keystore
        let master_password = config_manager.get_master_password()?;
        let keystore_path = config_manager.get_keystore_path();
        let keystore = if is_initialized(&keystore_path) {
            KeyStore::unlock(&keystore_path, &master_password)?
        } else {
            let keystore = KeyStore::initialize(&keystore_path, &master_password)?;
            if let Some(recovery_key) = &keystore.recovery_key {
                println!("🔑 Recovery Key (save securely): {}", recovery_key);
            }
            keystore
        };

        // Initialize crypto manager
        let mut crypto = CryptoManager::new();
        crypto.initialize_with_key(keystore.dek);

        // Encrypt the mnemonic
        let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;

        // Create stored record
        let record = StoredRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Mnemonic,
            encrypted_data,
            nonce,
            tags: vec!["crypto".to_string(), "wallet".to_string(), "mnemonic".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Get database path and save
        let db_config = config_manager.get_database_config()?;
        let db_path = PathBuf::from(db_config.path);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save to database
        let mut vault = Vault::open(&db_path, &master_password)?;
        vault.add_record(&record)?;

        println!("✅ Mnemonic saved to database as '{}'", name);
    }

    println!("🎯 Mnemonic: {}", mnemonic);
    println!("⚠️  Warning: Save this mnemonic in a secure location! Never share it!");

    Ok(())
}

async fn validate_mnemonic(words: &str) -> Result<()> {
    let is_valid = bip39::validate_mnemonic(words)?;

    if is_valid {
        println!("✅ Mnemonic is valid");
    } else {
        println!("❌ Mnemonic is invalid");
    }

    Ok(())
}
