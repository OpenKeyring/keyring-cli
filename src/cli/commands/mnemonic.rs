use clap::Parser;
use crate::cli::ConfigManager;
use crate::crypto::CryptoManager;
use crate::error::{KeyringError, Result};
use crate::db::models::{Record, RecordType};

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
    let mut config = ConfigManager::new()?;
    let crypto_config = config.get_crypto_config()?;
    let mut crypto = CryptoManager::new(&crypto_config);

    if let Some(word_count) = args.generate {
        generate_mnemonic(&mut crypto, word_count, args.name).await?;
    } else if let Some(words) = args.validate {
        validate_mnemonic(&mut crypto, &words).await?;
    } else {
        println!("Please specify either --generate or --validate");
    }

    Ok(())
}

async fn generate_mnemonic(crypto: &mut CryptoManager, word_count: u8, name: Option<String>) -> Result<()> {
    let mnemonic = crypto.generate_mnemonic(word_count)?;

    if let Some(name) = name {
        // Save as a mnemonic record
        let master_password = ""; // This would come from config
        let record = Record {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Mnemonic,
            encrypted_data: crypto.encrypt(&mnemonic, master_password)?,
            name,
            username: None,
            url: None,
            notes: Some("Cryptocurrency wallet mnemonic".to_string()),
            tags: vec!["crypto".to_string(), "wallet".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Save to database
        let mut config = ConfigManager::new()?;
        let mut db = crate::db::DatabaseManager::new(&config.get_database_config()?).await?;
        db.create_record(&record).await?;

        println!("✅ Mnemonic generated and saved as '{}'", record.name);
    }

    println!("🎯 Mnemonic: {}", mnemonic);
    println!("⚠️  Warning: Save this mnemonic in a secure location! Never share it!");

    Ok(())
}

async fn validate_mnemonic(crypto: &mut CryptoManager, words: &str) -> Result<()> {
    let is_valid = crypto.validate_mnemonic(words)?;

    if is_valid {
        println!("✅ Mnemonic is valid");
    } else {
        println!("❌ Mnemonic is invalid");
    }

    Ok(())
}