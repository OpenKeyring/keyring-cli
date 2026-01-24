use clap::Parser;
use crate::cli::ConfigManager;
use crate::crypto::{CryptoManager, KeyDerivation};
use crate::error::{KeyringError, Result};
use crate::db::models::{Record, RecordType};

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[clap(short, long)]
    pub name: String,
    #[clap(short, long, default_value = "16")]
    pub length: usize,
    #[clap(short, long)]
    pub memorable: bool,
    #[clap(short, long)]
    pub pin: bool,
    #[clap(short, long, default_value = "password")]
    pub r#type: String,
    #[clap(long, short)]
    pub tags: Vec<String>,
    #[clap(long)]
    pub sync: bool,
}

pub async fn generate_password(args: GenerateArgs) -> Result<()> {
    let mut config = ConfigManager::new()?;
    let mut crypto = CryptoManager::new(&config.get_crypto_config()?);

    // Generate password based on type
    let password = if args.memorable {
        crypto.generate_memorable_password(args.length, 4)?
    } else if args.pin {
        crypto.generate_pin(args.length)?
    } else {
        crypto.generate_random_password(args.length)?
    };

    // Create record
    let record = Record {
        id: uuid::Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: crypto.encrypt(&password, &config.get_master_password()?)?,
        name: args.name,
        username: None,
        url: None,
        notes: Some(if args.memorable { "Memorable password" } else if args.pin { "PIN code" } else { "Random password" }.to_string()),
        tags: args.tags,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Save to database
    let mut db = crate::db::DatabaseManager::new(&config.get_database_config()?).await?;
    db.create_record(&record).await?;

    if args.sync {
        // Sync the record
        sync_record(&config, &record).await?;
    }

    // Copy to clipboard
    let mut clipboard = crate::clipboard::ClipboardService::new(
        create_platform_clipboard()?,
        crate::clipboard::ClipboardConfig::default(),
    );
    clipboard.copy_password(&password)?;

    println!("✅ Password generated and copied to clipboard");
    println!("📋 Password: {}", password);

    Ok(())
}

async fn sync_record(config: &ConfigManager, record: &Record) -> Result<()> {
    // Implementation for syncing
    println!("🔄 Syncing record...");
    Ok(())
}

fn create_platform_clipboard() -> Result<Box<dyn crate::clipboard::ClipboardManager>> {
    match std::env::consts::OS {
        "macos" => Ok(Box::new(crate::clipboard::macos::MacOSClipboard)),
        "linux" => Ok(Box::new(crate::clipboard::linux::LinuxClipboard)),
        "windows" => Ok(Box::new(crate::clipboard::windows::WindowsClipboard)),
        _ => Err(KeyringError::UnsupportedPlatform),
    }
}