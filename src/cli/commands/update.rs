use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::error::{KeyringError, Result};

#[derive(Parser, Debug)]
pub struct UpdateArgs {
    pub name: String,
    #[clap(short, long)]
    pub password: Option<String>,
    #[clap(short, long)]
    pub username: Option<String>,
    #[clap(short, long)]
    pub url: Option<String>,
    #[clap(short, long)]
    pub notes: Option<String>,
    #[clap(short, long)]
    pub tags: Vec<String>,
    #[clap(long)]
    pub sync: bool,
}

pub async fn update_record(args: UpdateArgs) -> Result<()> {
    let mut config = ConfigManager::new()?;
    let mut db = DatabaseManager::new(&config.get_database_config()?).await?;

    let mut record = match db.find_record_by_name(&args.name).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(KeyringError::RecordNotFound(args.name)),
        Err(e) => return Err(e),
    };

    // Update fields if provided
    if let Some(username) = args.username {
        record.username = Some(username);
    }
    if let Some(url) = args.url {
        record.url = Some(url);
    }
    if let Some(notes) = args.notes {
        record.notes = Some(notes);
    }
    if !args.tags.is_empty() {
        record.tags = args.tags;
    }

    if let Some(new_password) = args.password {
        let master_password = config.get_master_password()?;
        let crypto_config = config.get_crypto_config()?;
        let mut crypto = crate::crypto::CryptoManager::new(&crypto_config);
        record.encrypted_data = crypto.encrypt(&new_password, &master_password)?;
    }

    record.updated_at = chrono::Utc::now();

    db.update_record(&record).await?;

    if args.sync {
        sync_record(&config, &record).await?;
    }

    println!("✅ Record updated successfully");

    Ok(())
}

async fn sync_record(_config: &ConfigManager, _record: &crate::db::models::DecryptedRecord) -> Result<()> {
    println!("🔄 Syncing record...");
    Ok(())
}