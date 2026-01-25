use anyhow::Result;
use clap::{Parser, Subcommand};
use keyring_cli::cli::commands;

/// OpenKeyring CLI - A privacy-first password manager
#[derive(Parser, Debug)]
#[command(name = "ok")]
#[command(author = "OpenKeyring Team")]
#[command(version = "0.1.0")]
#[command(about = "OpenKeyring CLI - A privacy-first, local-first password manager", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Path to config file
    #[arg(long, global = true)]
    config: Option<String>,

    /// Path to database file
    #[arg(short, long, global = true)]
    database: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a new password
    #[command(alias = "gen")]
    Generate {
        /// Password name (required)
        #[arg(short, long)]
        name: String,

        /// Password length (default: 16)
        #[arg(short, long, default_value = "16")]
        length: usize,

        /// Include numbers
        #[arg(long, default_value = "true")]
        numbers: bool,

        /// Include symbols
        #[arg(long, default_value = "true")]
        symbols: bool,

        /// Generate memorable password (word-based)
        #[arg(long, short)]
        memorable: bool,

        /// Number of words for memorable password
        #[arg(long, default_value = "4")]
        words: usize,

        /// Generate PIN code
        #[arg(long, short)]
        pin: bool,

        /// Username
        #[arg(short, long)]
        username: Option<String>,

        /// Website URL
        #[arg(long)]
        url: Option<String>,

        /// Notes
        #[arg(long)]
        notes: Option<String>,

        /// Tags (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Copy to clipboard after generation
        #[arg(long, short)]
        copy: bool,

        /// Sync after generation
        #[arg(long)]
        sync: bool,
    },

    /// List all passwords
    #[command(alias = "ls")]
    List {
        /// Filter by type
        #[arg(short, long, value_parser = ["password", "ssh_key", "api_credential", "mnemonic", "private_key"])]
        r#type: Option<String>,

        /// Filter by tags (AND logic)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Filter by tag (OR logic, can be used multiple times)
        #[arg(long)]
        tag: Vec<String>,

        /// Sort by field
        #[arg(long, value_parser = ["name", "updated_at", "created_at"])]
        sort: Option<String>,

        /// Reverse sort order
        #[arg(long)]
        reverse: bool,

        /// Output format
        #[arg(short, long, value_parser = ["text", "json", "yaml"])]
        output: Option<String>,
    },

    /// Show password details
    #[command(alias = "get")]
    Show {
        /// Password name or ID
        name: String,

        /// Show password (default: hidden)
        #[arg(long, short)]
        password: bool,

        /// Copy password to clipboard
        #[arg(long, short)]
        copy: bool,

        /// Clipboard timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Show specific field only
        #[arg(long)]
        field: Option<String>,

        /// Show history
        #[arg(long)]
        history: bool,
    },

    /// Update password record
    #[command(alias = "edit")]
    Update {
        /// Password name or ID
        name: String,

        /// New password
        #[arg(short, long)]
        password: Option<String>,

        /// New username
        #[arg(short, long)]
        username: Option<String>,

        /// New URL
        #[arg(long)]
        url: Option<String>,

        /// New notes
        #[arg(long)]
        notes: Option<String>,

        /// New tags (replaces existing)
        #[arg(short, long, value_delimiter = ',')]
        tags: Option<Vec<String>>,

        /// Add tags
        #[arg(long, value_delimiter = ',')]
        add_tags: Option<Vec<String>>,

        /// Remove tags
        #[arg(long, value_delimiter = ',')]
        remove_tags: Option<Vec<String>>,

        /// Sync after update
        #[arg(long)]
        sync: bool,
    },

    /// Delete password record
    #[command(alias = "rm")]
    Delete {
        /// Password name or ID
        name: String,

        /// Sync deletion to cloud
        #[arg(long)]
        sync: bool,

        /// Force delete without confirmation
        #[arg(long, short)]
        force: bool,
    },

    /// Search passwords
    #[command(alias = "find")]
    Search {
        /// Search query
        query: String,

        /// Filter by type
        #[arg(short, long)]
        r#type: Option<String>,

        /// Output format
        #[arg(short, long, value_parser = ["text", "json", "yaml"])]
        output: Option<String>,
    },

    /// Sync passwords to cloud
    Sync {
        /// Preview changes without executing
        #[arg(long)]
        dry_run: bool,

        /// Force full sync (ignore incremental cache)
        #[arg(long)]
        full: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show sync status
    #[command(alias = "status")]
    SyncStatus,

    /// Manage trusted devices
    Devices {
        #[command(subcommand)]
        device_command: DeviceCommands,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },

    /// Check password health
    #[command(alias = "check")]
    Health {
        /// Check for leaked passwords (HIBP API)
        #[arg(long)]
        leaks: bool,

        /// Check for weak passwords
        #[arg(long)]
        weak: bool,

        /// Check for duplicate passwords
        #[arg(long)]
        duplicate: bool,

        /// Run all checks
        #[arg(long, short)]
        all: bool,
    },

    /// Mnemonic operations
    #[command(alias = "mn")]
    Mnemonic {
        #[command(subcommand)]
        mnemonic_command: MnemonicCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DeviceCommands {
    /// List all trusted devices
    List,

    /// Remove a trusted device
    Remove {
        /// Device ID to remove
        device_id: String,

        /// Force removal without confirmation
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// List all configuration
    List,

    /// Reset configuration to defaults
    Reset {
        /// Confirm reset
        #[arg(long, short)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum MnemonicCommands {
    /// Generate a new mnemonic
    #[command(alias = "gen")]
    Generate {
        /// Number of words (12 or 24)
        #[arg(short, long, value_parser = clap::value_parser!(u8).range(12..=24))]
        words: Option<u8>,

        /// Language (english or chinese)
        #[arg(short, long, value_parser = ["english", "chinese"])]
        language: Option<String>,

        /// Name for the mnemonic
        #[arg(short, long)]
        name: Option<String>,

        /// Passphrase hint
        #[arg(long)]
        hint: Option<String>,
    },

    /// Validate a mnemonic phrase
    #[command(alias = "val")]
    Validate {
        /// Mnemonic words (space or comma-separated)
        words: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbose flag
    setup_logging(cli.verbose, cli.quiet);

    // Execute command
    match cli.command {
        Commands::Generate {
            name,
            length,
            numbers,
            symbols,
            memorable,
            words,
            pin,
            username,
            url,
            notes,
            tags,
            copy,
            sync,
        } => {
            use cli::commands::generate::GenerateArgs;
            let args = GenerateArgs {
                name,
                length,
                numbers,
                symbols,
                memorable,
                words,
                pin,
                username,
                url,
                notes,
                tags,
                copy,
                sync,
            };
            commands::generate::execute(args).await?
        }

        Commands::List {
            r#type,
            tags,
            tag,
            sort,
            reverse,
            output,
        } => commands::list::execute(r#type, tags, tag, sort, reverse, output).await?,

        Commands::Show {
            name,
            password,
            copy,
            timeout,
            field,
            history,
        } => commands::show::execute(name, password, copy, timeout, field, history).await?,

        Commands::Update {
            name,
            password,
            username,
            url,
            notes,
            tags,
            add_tags,
            remove_tags,
            sync,
        } => {
            commands::update::execute(
                name,
                password,
                username,
                url,
                notes,
                tags,
                add_tags,
                remove_tags,
                sync,
            )
            .await?
        }

        Commands::Delete { name, sync, force } => {
            commands::delete::execute(name, sync, force).await?
        }

        Commands::Search {
            query,
            r#type,
            output,
        } => commands::search::execute(query, r#type, output).await?,

        Commands::Sync {
            dry_run,
            full,
            verbose,
        } => commands::sync::execute(dry_run, full, verbose).await?,

        Commands::SyncStatus => commands::sync::execute_status().await?,

        Commands::Devices { device_command } => commands::devices::execute(device_command).await?,

        Commands::Config { config_command } => commands::config::execute(config_command).await?,

        Commands::Health {
            leaks,
            weak,
            duplicate,
            all,
        } => commands::health::execute(leaks, weak, duplicate, all).await?,

        Commands::Mnemonic { mnemonic_command } => {
            commands::mnemonic::execute(mnemonic_command).await?
        }
    }

    Ok(())
}

fn setup_logging(verbose: bool, quiet: bool) {
    use log::LevelFilter;

    let level = if quiet {
        LevelFilter::Error
    } else if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    // Simple env logger setup
    env_logger::Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "[{} {}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
