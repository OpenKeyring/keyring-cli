use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::error::{KeyringError, Result};

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

    let mut config = ConfigManager::new()?;
    let mut db = DatabaseManager::new(&config.get_database_config()?).await?;

    match db.find_record_by_name(&args.name).await {
        Ok(Some(record)) => {
            db.delete_record(&record.id).await?;

            if args.sync {
                sync_deletion(&config, &record.id).await?;
            }

            println!("✅ Record '{}' deleted successfully", args.name);
        }
        Ok(None) => {
            return Err(KeyringError::RecordNotFound(args.name));
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

async fn sync_deletion(config: &ConfigManager, record_id: &uuid::Uuid) -> Result<()> {
    println!("🔄 Syncing deletion...");
    Ok(())
}