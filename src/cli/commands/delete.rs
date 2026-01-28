use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::{Error, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct DeleteArgs {
    pub name: String,
    #[clap(long, short)]
    pub confirm: bool,
    #[clap(long)]
    pub sync: bool,
}

pub async fn delete_record(args: DeleteArgs) -> Result<()> {
    if !args.confirm {
        println!("❌ Deletion requires explicit confirmation with --confirm");
        return Ok(());
    }

    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Open vault
    let mut vault = Vault::open(&db_path, "")?;

    // Find record by name
    let record = match vault.find_record_by_name(&args.name)? {
        Some(r) => r,
        None => {
            return Err(Error::RecordNotFound {
                name: args.name.clone(),
            });
        }
    };

    println!("🗑️  Deleting record: {}", args.name);

    // Delete the record using its UUID
    vault.delete_record(&record.id.to_string())?;

    if args.sync {
        sync_deletion(&config, &record.id.to_string()).await?;
    }

    println!("✅ Record '{}' deleted successfully", args.name);
    Ok(())
}

async fn sync_deletion(_config: &ConfigManager, _record_id: &str) -> Result<()> {
    println!("🔄 Syncing deletion...");
    Ok(())
}
