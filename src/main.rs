use clap::Parser;
use std::process;

// CLI commands
mod commands;
mod config;
mod utils;

use commands::*;
use config::ConfigManager;

#[derive(Parser)]
#[command(
    name = "ok",
    about = "OpenKeyring - Privacy-first password manager",
    version = "0.1.0",
    author = "OpenKeyring Team"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate a new password
    #[command(about = "Generate a new password")]
    Generate {
        #[clap(short, long)]
        name: String,
        #[clap(short, long, default_value = "16")]
        length: usize,
        #[clap(short, long)]
        memorable: bool,
        #[clap(short, long)]
        pin: bool,
        #[clap(short, long)]
        r#type: String,
        #[clap(long, short)]
        tags: Vec<String>,
        #[clap(long)]
        sync: bool,
    },
    /// List all password records
    #[command(about = "List all password records")]
    List {
        #[clap(short, long)]
        r#type: Option<String>,
        #[clap(short, long)]
        tags: Vec<String>,
        #[clap(short, long)]
        limit: Option<usize>,
    },
    /// Show a password record
    #[command(about = "Show a password record")]
    Show {
        name: String,
        #[clap(short, long)]
        show_password: bool,
        #[clap(short, long)]
        copy_to_clipboard: bool,
        #[clap(long)]
        sync: bool,
    },
    /// Update a password record
    #[command(about = "Update a password record")]
    Update {
        name: String,
        #[clap(short, long)]
        password: Option<String>,
        #[clap(short, long)]
        username: Option<String>,
        #[clap(short, long)]
        url: Option<String>,
        #[clap(short, long)]
        notes: Option<String>,
        #[clap(short, long)]
        tags: Vec<String>,
        #[clap(long)]
        sync: bool,
    },
    /// Delete a password record
    #[command(about = "Delete a password record")]
    Delete {
        name: String,
        #[clap(long, short)]
        confirm: bool,
        #[clap(long)]
        sync: bool,
    },
    /// Search for password records
    #[command(about = "Search for password records")]
    Search {
        query: String,
        #[clap(short, long)]
        r#type: Option<String>,
        #[clap(short, long)]
        tags: Vec<String>,
        #[clap(short, long)]
        limit: Option<usize>,
    },
    /// Sync records across devices
    #[command(about = "Sync records across devices")]
    Sync {
        #[clap(long, short)]
        dry_run: bool,
        #[clap(long, short)]
        full: bool,
        #[clap(long, short)]
        status: bool,
        #[clap(long)]
        provider: Option<String>,
    },
    /// Check password health
    #[command(about = "Check password health")]
    Health {
        #[clap(long, short)]
        leaks: bool,
        #[clap(long, short)]
        weak: bool,
        #[clap(long, short)]
        duplicate: bool,
    },
    /// Manage devices
    #[command(about = "Manage connected devices")]
    Devices {
        #[clap(long, short)]
        remove: Option<String>,
    },
    /// Generate or validate mnemonic
    #[command(about = "Generate or validate mnemonic")]
    Mnemonic {
        #[clap(long, short)]
        generate: Option<u8>,
        #[clap(long, short)]
        validate: Option<String>,
        #[clap(long, short)]
        name: Option<String>,
    },
    /// Configuration management
    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },
    /// List all configuration values
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Handle unimplemented commands gracefully
    match handle_command(cli).await {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

async fn handle_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Generate { name, length, memorable, pin, r#type, tags, sync } => {
            generate_password(GenerateArgs {
                name,
                length,
                memorable,
                pin,
                r#type,
                tags,
                sync,
            }).await?;
        }
        Commands::List { r#type, tags, limit } => {
            list_records(ListArgs {
                r#type,
                tags,
                limit,
            }).await?;
        }
        Commands::Show { name, show_password, copy_to_clipboard, sync } => {
            show_record(ShowArgs {
                name,
                show_password,
                copy_to_clipboard,
                sync,
            }).await?;
        }
        Commands::Update { name, password, username, url, notes, tags, sync } => {
            update_record(UpdateArgs {
                name,
                password,
                username,
                url,
                notes,
                tags,
                sync,
            }).await?;
        }
        Commands::Delete { name, confirm, sync } => {
            delete_record(DeleteArgs {
                name,
                confirm,
                sync,
            }).await?;
        }
        Commands::Search { query, r#type, tags, limit } => {
            search_records(SearchArgs {
                query,
                r#type,
                tags,
                limit,
            }).await?;
        }
        Commands::Sync { dry_run, full, status, provider } => {
            sync_records(SyncArgs {
                dry_run,
                full,
                status,
                provider,
            }).await?;
        }
        Commands::Health { leaks, weak, duplicate } => {
            check_health(HealthArgs {
                leaks,
                weak,
                duplicate,
            }).await?;
        }
        Commands::Devices { remove } => {
            manage_devices(DevicesArgs {
                remove,
            }).await?;
        }
        Commands::Mnemonic { generate, validate, name } => {
            handle_mnemonic(MnemonicArgs {
                generate,
                validate,
                name,
            }).await?;
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Set { key, value } => {
                    set_config(key, value).await?;
                }
                ConfigAction::List => {
                    list_config().await?;
                }
            }
        }
    }

    Ok(())
}

async fn set_config(key: String, value: String) -> anyhow::Result<()> {
    let mut config = ConfigManager::new()?;
    // Implementation for setting config
    println!("Set {} = {}", key, value);
    Ok(())
}

async fn list_config() -> anyhow::Result<()> {
    let config = ConfigManager::new()?;
    // Implementation for listing config
    println!("Current configuration:");
    println!("Database path: {:?}", config.get_database_config()?);
    println!("Crypto config: {:?}", config.get_crypto_config()?);
    Ok(())
}
