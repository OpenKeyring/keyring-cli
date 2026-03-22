use crate::cli::ConfigManager;
use crate::db::Vault;
use crate::error::Result;
use crate::sync::conflict::ConflictResolution;
use crate::sync::service::SyncService;
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "sync")]
#[command(about = "Sync passwords to cloud storage", long_about = None)]
pub struct SyncCommand {
    /// Show sync status instead of syncing
    #[arg(long, short)]
    pub status: bool,

    /// Configure cloud storage provider
    #[arg(long, short)]
    pub config: bool,

    /// Cloud storage provider (for use with --config)
    #[arg(long)]
    pub provider: Option<String>,

    /// Direction: up, down, or both
    #[arg(short, long, default_value = "both")]
    pub direction: String,

    /// Dry run without making changes
    #[arg(long)]
    pub dry_run: bool,
}

impl SyncCommand {
    pub fn execute(&self) -> Result<()> {
        if self.status {
            println!("Sync status:");
            return Ok(());
        }

        if self.config {
            println!("Configuring provider: {:?}", self.provider);
            return Ok(());
        }

        println!("Syncing {} (dry run: {})", self.direction, self.dry_run);
        Ok(())
    }
}

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
    let config = ConfigManager::new()?;

    // Handle config flag for provider configuration
    if args.status {
        if let Some(provider) = &args.provider {
            return configure_provider(&config, provider);
        }
        // Show current sync configuration
        return show_sync_config(&config);
    }

    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    let sync_config = config.get_sync_config()?;
    let sync_dir = PathBuf::from(&sync_config.remote_path);

    // Get conflict resolution from config for sync
    let conflict_resolution = match sync_config.conflict_resolution.as_str() {
        "newer" => ConflictResolution::Newer,
        "older" => ConflictResolution::Older,
        "local" => ConflictResolution::Local,
        "remote" => ConflictResolution::Remote,
        _ => ConflictResolution::Newer,
    };

    if args.dry_run {
        let vault = Vault::open(&db_path, "")?;
        perform_dry_run(&vault, &sync_dir).await?;
        return Ok(());
    }

    // For actual sync, we need mutable vault
    let mut vault = Vault::open(&db_path, "")?;
    perform_sync(&mut vault, &sync_dir, conflict_resolution).await
}

async fn perform_dry_run(vault: &Vault, sync_dir: &Path) -> Result<()> {
    let pending = vault.get_pending_records()?;

    if pending.is_empty() {
        println!("🔍 Dry run - no pending records to sync");
        return Ok(());
    }

    // Calculate total size
    let total_size: usize = pending.iter().map(|r| r.encrypted_data.len()).sum();
    let size_kb = total_size / 1024;

    println!("🔍 Dry run - pending records:");
    println!("   Records to sync: {}", pending.len());
    println!("   Estimated size: {} KB", size_kb);
    println!("   Target: {}", sync_dir.display());

    Ok(())
}

async fn perform_sync(
    vault: &mut Vault,
    sync_dir: &Path,
    conflict_resolution: ConflictResolution,
) -> Result<()> {
    let sync_service = SyncService::new();

    println!("🔄 Starting sync...");
    println!("   Target: {}", sync_dir.display());
    println!("   Conflict resolution: {:?}", conflict_resolution);

    // Export pending records
    let exported = sync_service.export_pending_records(vault, sync_dir)?;
    if !exported.is_empty() {
        println!("   Exported {} pending records", exported.len());
    }

    // Import records from sync directory
    let stats = sync_service.import_from_directory(vault, sync_dir, conflict_resolution)?;

    println!(
        "   Imported: {}, Updated: {}, Resolved: {}",
        stats.imported, stats.updated, stats.conflicts
    );
    println!("✅ Sync completed");

    Ok(())
}

fn configure_provider(_config: &ConfigManager, provider: &str) -> Result<()> {
    println!("⚙️  Configuring cloud storage provider: {}", provider);

    let valid_providers = [
        "icloud",
        "dropbox",
        "gdrive",
        "onedrive",
        "webdav",
        "sftp",
        "aliyundrive",
        "oss",
    ];

    if !valid_providers.contains(&provider) {
        return Err(crate::error::KeyringError::InvalidInput {
            context: format!(
                "Invalid provider. Valid options: {}",
                valid_providers.join(", ")
            ),
        });
    }

    println!("✓ Provider set to: {}", provider);
    println!("ℹ️  Use 'ok config set sync.remote_path <path>' to set the remote path");
    println!("ℹ️  Use 'ok config set sync.enabled true' to enable sync");

    Ok(())
}

fn show_sync_config(config: &ConfigManager) -> Result<()> {
    let sync_config = config.get_sync_config()?;

    println!("⚙️  Sync Configuration:");
    println!("   Enabled: {}", sync_config.enabled);
    println!("   Provider: {}", sync_config.provider);
    println!("   Remote Path: {}", sync_config.remote_path);
    println!(
        "   Conflict Resolution: {}",
        sync_config.conflict_resolution
    );
    println!("   Auto Sync: {}", sync_config.auto_sync);

    Ok(())
}
