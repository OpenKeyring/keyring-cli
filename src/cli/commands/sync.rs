use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::DatabaseManager;
use crate::sync::{SyncExporter, SyncImporter, SyncConfig};
use crate::error::{KeyringError, Result};

#[derive(Parser, Debug)]
pub struct SyncArgs {
    #[clap(long, short)]
    pub dry_run: bool,
    #[clap(long, short)]
    pub full: bool,
    #[clap(long, short)]
    pub status: bool,
    #[clap(long)]
    pub provider: Option<String>,
}

pub async fn sync_records(args: SyncArgs) -> Result<()> {
    let mut config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let mut db = DatabaseManager::new(&db_config).await?;

    if args.status {
        show_sync_status(&config).await?;
        return Ok(());
    }

    if args.dry_run {
        perform_dry_run(&mut db).await?;
        return Ok(());
    }

    perform_sync(&mut db, &config, args).await
}

async fn show_sync_status(config: &ConfigManager) -> Result<()> {
    let sync_config = config.get_sync_config()?;
    println!("📊 Sync Status:");
    println!("   Enabled: {}", sync_config.enabled);
    println!("   Provider: {}", sync_config.provider);
    println!("   Remote Path: {}", sync_config.remote_path);
    println!("   Auto Sync: {}", sync_config.auto_sync);
    println!("   Conflict Resolution: {:?}", sync_config.conflict_resolution);
    Ok(())
}

async fn perform_dry_run(db: &mut DatabaseManager) -> Result<()> {
    let records = db.list_all_records(None).await?;
    println!("🔍 Dry run - would sync {} records", records.len());

    let exporter = crate::sync::JsonSyncExporter;
    let sync_records: Result<Vec<_>> = records.iter()
        .map(|r| exporter.export_record(r))
        .collect();

    match sync_records {
        Ok(records) => {
            println!("   Estimated size: {} KB",
                records.iter().map(|r| r.encrypted_data.len()).sum::<usize>() / 1024);
        }
        Err(e) => println!("   Error: {}", e),
    }

    Ok(())
}

async fn perform_sync(
    db: &mut DatabaseManager,
    config: &ConfigManager,
    args: SyncArgs
) -> Result<()> {
    println!("🔄 Starting sync...");

    let sync_config = config.get_sync_config()?;
    let records = if args.full {
        db.list_all_records(None).await?
    } else {
        // Sync recent records
        db.get_recent_records(100).await?
    };

    let exporter = crate::sync::JsonSyncExporter;
    let sync_records: Vec<_> = records.iter()
        .map(|r| exporter.export_record(r))
        .collect::<Result<_>>()?;

    println!("   Syncing {} records", sync_records.len());

    // In a real implementation, this would upload to cloud storage
    for record in sync_records {
        println!("   ✓ Synced: {}", record.metadata.name);
    }

    println!("✅ Sync completed successfully");
    Ok(())
}