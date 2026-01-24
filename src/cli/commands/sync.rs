use clap::Parser;
use crate::cli::ConfigManager;
use crate::db::{DatabaseManager, vault::Vault};
use crate::sync::{SyncService, ConflictResolution};
use crate::error::{KeyringError, Result};
use std::path::PathBuf;

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
    let mut db = DatabaseManager::new(&db_config.path)?;
    db.open()?;

    // Get vault from database connection
    let conn = db.connection_mut()?;
    let mut vault = Vault { conn };

    if args.status {
        show_sync_status(&vault).await?;
        return Ok(());
    }

    let sync_config = config.get_sync_config()?;
    let sync_dir = PathBuf::from(&sync_config.remote_path);
    let conflict_resolution = match sync_config.conflict_resolution.as_str() {
        "newer" => ConflictResolution::Newer,
        "older" => ConflictResolution::Older,
        "local" => ConflictResolution::Local,
        "remote" => ConflictResolution::Remote,
        _ => ConflictResolution::Newer,
    };

    if args.dry_run {
        perform_dry_run(&vault, &sync_dir).await?;
        return Ok(());
    }

    perform_sync(&mut vault, &sync_dir, conflict_resolution).await
}

async fn show_sync_status(vault: &Vault) -> Result<()> {
    let sync_service = SyncService::new();
    let status = sync_service.get_sync_status(vault)?;
    
    println!("📊 Sync Status:");
    println!("   Total records: {}", status.total);
    println!("   Pending: {}", status.pending);
    println!("   Conflicts: {}", status.conflicts);
    println!("   Synced: {}", status.synced);
    Ok(())
}

async fn perform_dry_run(vault: &Vault, sync_dir: &PathBuf) -> Result<()> {
    let sync_service = SyncService::new();
    let pending = sync_service.get_pending_records(vault)?;
    
    println!("🔍 Dry run - would sync {} records", pending.len());
    
    if !pending.is_empty() {
        let exported = sync_service.export_pending_records(vault, sync_dir)?;
        let total_size: usize = exported.iter()
            .map(|r| r.encrypted_data.len())
            .sum();
        println!("   Estimated size: {} KB", total_size / 1024);
        println!("   Files would be written to: {}", sync_dir.display());
    }
    
    Ok(())
}

async fn perform_sync(
    vault: &mut Vault,
    sync_dir: &PathBuf,
    conflict_resolution: ConflictResolution,
) -> Result<()> {
    println!("🔄 Starting sync...");

    let sync_service = SyncService::new();

    // Export pending records
    let exported = sync_service.export_pending_records(vault, sync_dir)?;
    println!("   Exported {} records to {}", exported.len(), sync_dir.display());

    // Import from directory
    let stats = sync_service.import_from_directory(vault, sync_dir, conflict_resolution)?;
    
    println!("   Imported: {} new records", stats.imported);
    println!("   Updated: {} existing records", stats.updated);
    if stats.conflicts > 0 {
        println!("   Resolved: {} conflicts", stats.conflicts);
    }

    println!("✅ Sync completed successfully");
    Ok(())
}