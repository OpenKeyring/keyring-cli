use anyhow::Result;
use clap::{Parser, Subcommand};

/// OpenKeyring CLI - A privacy-first password manager
#[derive(Parser, Debug)]
#[command(name = "ok")]
#[command(author = "OpenKeyring Team")]
#[command(version = "0.1.0")]
#[command(about = "OpenKeyring CLI - A privacy-first, local-first password manager", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to database file
    #[arg(short, long, global = true)]
    database: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run onboarding wizard for first-time setup
    #[command(alias = "init")]
    Wizard,

    /// Recover vault using Passkey
    #[command(alias = "restore")]
    Recover {
        /// 24-word Passkey (optional, will prompt if not provided)
        #[arg(long, short)]
        passkey: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands or launch TUI
    match cli.command {
        None => {
            // No command: launch TUI mode
            keyring_cli::tui::run_tui().map_err(|e| anyhow::anyhow!("TUI error: {}", e))
        }
        Some(Commands::Wizard) => {
            use keyring_cli::cli::commands::wizard::WizardArgs;
            let args = WizardArgs {};
            keyring_cli::cli::commands::wizard::run_wizard(args)
                .await
                .map_err(|e| anyhow::anyhow!("Wizard error: {}", e))
        }
        Some(Commands::Recover { passkey }) => {
            use keyring_cli::cli::commands::recover::RecoverArgs;
            let args = RecoverArgs { passkey };
            keyring_cli::cli::commands::recover::execute(args)
                .await
                .map_err(|e| anyhow::anyhow!("Recover error: {}", e))
        }
    }
}
