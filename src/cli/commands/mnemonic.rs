use clap::Parser;
use crate::error::Result;
use crate::db::models::{DecryptedRecord, RecordType};
use crate::crypto::bip39;

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
        // Create a record placeholder for display purposes
        let record = DecryptedRecord {
            id: uuid::Uuid::new_v4(),
            record_type: RecordType::Mnemonic,
            name,
            username: None,
            password: mnemonic.clone(),
            url: None,
            notes: Some("Cryptocurrency wallet mnemonic".to_string()),
            tags: vec!["crypto".to_string(), "wallet".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // TODO: Save to database - requires proper encryption and storage
        // For now, just display the mnemonic
        println!("✅ Mnemonic generated as '{}'", record.name);
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
