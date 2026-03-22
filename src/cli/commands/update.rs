use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::{Error, Result};
use clap::Parser;
use std::path::PathBuf;

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
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Open vault
    let mut vault = Vault::open(&db_path, "")?;

    // Find record by name
    let mut record = match vault.find_record_by_name(&args.name)? {
        Some(r) => r,
        None => {
            return Err(Error::RecordNotFound {
                name: args.name.clone(),
            });
        }
    };

    println!("🔄 Updating record: {}", args.name);

    // Parse existing encrypted data as JSON
    let mut payload: serde_json::Value =
        serde_json::from_slice(&record.encrypted_data).map_err(|e| Error::InvalidInput {
            context: format!("Failed to parse record data: {}", e),
        })?;

    // Update fields
    if let Some(password) = args.password {
        println!("   - Password: ***");
        payload["password"] = serde_json::json!(password);
    }
    if let Some(username) = args.username {
        println!("   - Username: {}", username);
        payload["username"] = serde_json::json!(username);
    }
    if let Some(url) = args.url {
        println!("   - URL: {}", url);
        payload["url"] = serde_json::json!(url);
    }
    if let Some(notes) = args.notes {
        println!("   - Notes: {}", notes);
        payload["notes"] = serde_json::json!(notes);
    }
    if !args.tags.is_empty() {
        println!("   - Tags: {}", args.tags.join(", "));
        payload["tags"] = serde_json::json!(args.tags);
        record.tags = args.tags.clone();
    }

    // Set updated timestamp
    record.updated_at = chrono::Utc::now();

    // Re-serialize the payload
    record.encrypted_data = serde_json::to_vec(&payload)?;

    // Update the record in the database
    vault.update_record(&record)?;

    println!("✅ Record '{}' updated successfully", args.name);

    if args.sync {
        sync_record(&config).await?;
    }

    Ok(())
}

async fn sync_record(_config: &ConfigManager) -> Result<()> {
    println!("🔄 Syncing record...");
    Ok(())
}
