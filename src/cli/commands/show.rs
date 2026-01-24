use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::error::{KeyringError, Result};
use crate::cli::utils::PrettyPrinter;

#[derive(Parser, Debug)]
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
    let mut config = ConfigManager::new()?;
    let mut db = DatabaseManager::new(&config.get_database_config()?).await?;

    let mut record = match db.find_record_by_name(&args.name).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(KeyringError::RecordNotFound(args.name)),
        Err(e) => return Err(e),
    };

    // Decrypt password if requested
    if args.show_password || args.copy_to_clipboard {
        let password = decrypt_password(&mut config, &mut db, &mut record).await?;

        if args.copy_to_clipboard {
            let mut clipboard = crate::clipboard::ClipboardService::new(
                create_platform_clipboard()?,
                crate::clipboard::ClipboardConfig::default(),
            );
            clipboard.copy_password(&password)?;
            println!("📋 Password copied to clipboard");
        } else if args.show_password {
            println!("🔑 Password: {}", password);
        }
    }

    PrettyPrinter::print_record(&record);

    if args.sync {
        sync_record(&config, &record).await?;
    }

    Ok(())
}

async fn decrypt_password(
    config: &mut ConfigManager,
    db: &mut DatabaseManager,
    record: &mut crate::db::models::Record,
) -> Result<String> {
    let master_password = config.get_master_password()?;
    let crypto_config = config.get_crypto_config()?;
    let mut crypto = crate::crypto::CryptoManager::new(&crypto_config);
    crypto.decrypt(&record.encrypted_data, &master_password)
}

async fn sync_record(config: &ConfigManager, record: &crate::db::models::Record) -> Result<()> {
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