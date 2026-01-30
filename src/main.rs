use anyhow::Result;
use clap::{Parser, Subcommand};
use keyring_cli::cli::{self, mcp};

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

    /// Disable TUI mode (force CLI mode)
    #[arg(long, global = true)]
    no_tui: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a new password
    #[command(alias = "generate")]
    New {
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
        #[arg(short = 't', long, value_parser = ["password", "ssh_key", "api_credential", "mnemonic", "private_key"])]
        r#type: Option<String>,

        /// Filter by tags (AND logic)
        #[arg(short = 'T', long, value_delimiter = ',')]
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

        /// Print password to terminal (WARNING: visible in command history, requires confirmation)
        #[arg(long, short)]
        print: bool,

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

        /// Configure cloud storage provider
        #[arg(long, short)]
        config: bool,

        /// Cloud storage provider (icloud, dropbox, gdrive, onedrive, webdav, sftp, aliyundrive, oss)
        #[arg(long)]
        provider: Option<String>,

        /// Sync direction: up, down, or both
        #[arg(short, long, default_value = "both")]
        direction: String,
    },

    /// Show sync status
    #[command(alias = "status")]
    SyncStatus,

    /// Manage MCP (Model Context Protocol) server
    Mcp {
        #[command(subcommand)]
        command: mcp_commands::MCPCommands,
    },

    /// Manage trusted devices
    Devices {
        #[command(subcommand)]
        device_command: DeviceCommands,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        config_command: commands::config::ConfigCommands,
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

    /// Manage keyboard shortcuts
    #[command(alias = "kb")]
    Keybindings {
        /// List all keyboard shortcuts
        #[arg(long, short)]
        list: bool,

        /// Validate keybindings configuration
        #[arg(long, short)]
        validate: bool,

        /// Reset keybindings to defaults
        #[arg(long, short)]
        reset: bool,

        /// Edit keybindings configuration
        #[arg(long, short)]
        edit: bool,
    },

    /// Recover vault using Passkey
    #[command(alias = "restore")]
    Recover {
        /// 24-word Passkey (optional, will prompt if not provided)
        #[arg(long, short)]
        passkey: Option<String>,
    },

    /// Run onboarding wizard for first-time setup
    #[command(alias = "init")]
    Wizard,

    /// MCP server management
    #[command(subcommand)]
    Mcp {
        #[command(subcommand)]
        command: keyring_cli::cli::mcp::MCPCommands,
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

    // Launch TUI if no command provided and TUI is not disabled
    if cli.command.is_none() {
        if cli.no_tui {
            // No command and --no-tui flag: show help
            println!("OpenKeyring CLI v0.1.0");
            println!("Use --help for usage information or run without --no-tui for interactive TUI mode.");
            return Ok(());
        } else {
            // No command: launch TUI mode
            return keyring_cli::tui::run_tui().map_err(|e| anyhow::anyhow!("TUI error: {}", e));
        }
    }

    // Execute command (CLI mode)
    match cli.command.unwrap() {
        Commands::New {
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
            use commands::generate::NewArgs;
            let args = NewArgs {
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
            tag: _,
            sort: _,
            reverse: _,
            output: _,
        } => {
            use commands::list::ListArgs;
            let args = ListArgs {
                r#type,
                tags,
                limit: None,
            };
            commands::list::list_records(args).await?
        }

        Commands::Show {
            name,
            print,
            copy,
            timeout,
            field,
            history,
        } => commands::show::execute(name, print, copy, timeout, field, history).await?,

        Commands::Update {
            name,
            password,
            username,
            url,
            notes,
            tags,
            add_tags: _,
            remove_tags: _,
            sync,
        } => {
            use commands::update::UpdateArgs;
            let args = UpdateArgs {
                name,
                password,
                username,
                url,
                notes,
                tags: tags.unwrap_or_default(),
                sync,
            };
            commands::update::update_record(args).await?
        }

        Commands::Delete { name, sync, force } => {
            use commands::delete::DeleteArgs;
            let args = DeleteArgs {
                name,
                confirm: force,
                sync,
            };
            commands::delete::delete_record(args).await?
        }

        Commands::Search {
            query,
            r#type,
            output: _,
        } => {
            use commands::search::SearchArgs;
            let args = SearchArgs {
                query,
                r#type,
                tags: vec![],
                limit: None,
            };
            commands::search::search_records(args).await?
        }

        Commands::Sync {
            dry_run,
            full,
            verbose: _,
            config,
            provider,
            direction: _,
        } => {
            use commands::sync::SyncArgs;
            let args = SyncArgs {
                dry_run,
                full,
                status: config,
                provider,
            };
            commands::sync::sync_records(args).await?
        }

        Commands::SyncStatus => {
            use commands::sync::SyncArgs;
            let args = SyncArgs {
                dry_run: false,
                full: false,
                status: true,
                provider: None,
            };
            commands::sync::sync_records(args).await?
        }

        Commands::Mcp { command } => {
            commands::mcp::handle_mcp_command(command).await?
        }

        Commands::Devices { device_command } => {
            use commands::devices::DevicesArgs;
            let args = match device_command {
                DeviceCommands::List => DevicesArgs { remove: None },
                DeviceCommands::Remove {
                    device_id,
                    force: _,
                } => DevicesArgs {
                    remove: Some(device_id),
                },
            };
            commands::devices::manage_devices(args).await?
        }

        Commands::Config { config_command } => commands::config::execute(config_command).await?,

        Commands::Health {
            leaks,
            weak,
            duplicate,
            all,
        } => {
            use commands::health::HealthArgs;
            let args = HealthArgs {
                leaks,
                weak,
                duplicate,
                all,
            };
            commands::health::check_health(args).await?
        }

        Commands::Mnemonic { mnemonic_command } => {
            use commands::mnemonic::MnemonicArgs;
            let args = match mnemonic_command {
                MnemonicCommands::Generate {
                    words,
                    language: _,
                    name,
                    hint: _,
                } => MnemonicArgs {
                    generate: words,
                    validate: None,
                    name,
                },
                MnemonicCommands::Validate { words } => MnemonicArgs {
                    generate: None,
                    validate: Some(words),
                    name: None,
                },
            };
            commands::mnemonic::handle_mnemonic(args).await?
        }

        Commands::Keybindings {
            list,
            validate,
            reset,
            edit,
        } => {
            use commands::keybindings::KeybindingsArgs;
            let args = KeybindingsArgs {
                list,
                validate,
                reset,
                edit,
            };
            commands::keybindings::manage_keybindings(args).await?
        }

        Commands::Recover { passkey } => {
            use commands::recover::RecoverArgs;
            let args = RecoverArgs { passkey };
            commands::recover::execute(args).await?
        }

        Commands::Wizard => {
            use keyring_cli::cli::commands::wizard::WizardArgs;
            let args = WizardArgs {};
            keyring_cli::cli::commands::wizard::run_wizard(args).await?
        }

        Commands::Mcp { command } => {
            mcp::handle_mcp_command(command).await?
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
