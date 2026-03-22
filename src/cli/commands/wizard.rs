//! CLI Wizard Command
//!
//! Interactive command-line wizard for first-time setup of OpenKeyring.

use crate::cli::ConfigManager;
use crate::crypto::passkey::Passkey;
use crate::error::Result;
use crate::onboarding::{initialize_keystore, is_initialized};
use anyhow::anyhow;

/// Wizard command arguments
#[derive(Debug, clap::Parser)]
pub struct WizardArgs {}

/// Run the onboarding wizard
pub async fn run_wizard(_args: WizardArgs) -> Result<()> {
    let config = ConfigManager::new()?;
    let keystore_path = config.get_keystore_path();

    if is_initialized(&keystore_path) {
        println!("✓ Already initialized");
        println!("  Keystore: {}", keystore_path.display());
        return Ok(());
    }

    println!("═══════════════════════════════════════════════════");
    println!("         OpenKeyring Initialization Wizard");
    println!("═══════════════════════════════════════════════════");
    println!();

    // Step 1: Welcome
    let choice = prompt_choice(
        "Choose setup method:",
        &[
            ("1", "New user (Generate new Passkey)"),
            ("2", "Import existing Passkey"),
        ],
    )?;

    let _passkey_words = if choice == "1" {
        // Generate new Passkey
        generate_new_passkey()?
    } else {
        // Import existing Passkey
        import_passkey()?
    };

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("         Set Master Password");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("💡 This password only encrypts the Passkey");
    println!("   Can be different from other devices");
    println!();

    // Step 3: Master password
    let password = prompt_password("Enter master password: ")?;
    let confirm = prompt_password("Confirm master password: ")?;

    if password != confirm {
        return Err(anyhow!("Passwords do not match").into());
    }

    if password.len() < 8 {
        return Err(anyhow!("Master password must be at least 8 characters").into());
    }

    // Initialize
    let keystore = initialize_keystore(&keystore_path, &password)
        .map_err(|e| anyhow!("Failed to initialize keystore: {}", e))?;

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("✓ Initialization Complete");
    println!("═══════════════════════════════════════════════════");
    println!("✓ Keystore: {}", keystore_path.display());
    println!(
        "✓ Recovery Key: {}",
        keystore
            .recovery_key
            .as_ref()
            .unwrap_or(&"(not generated)".to_string())
    );
    println!();
    println!("You can now start using OpenKeyring!");

    Ok(())
}

/// Generate a new Passkey
fn generate_new_passkey() -> Result<Vec<String>> {
    println!("Generating new Passkey...");

    let passkey = Passkey::generate(24)?;
    let words = passkey.to_words();

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("⚠️  PLEASE SAVE THE FOLLOWING 24 WORDS - THIS IS THE ONLY WAY TO RECOVER YOUR DATA!");
    println!("═══════════════════════════════════════════════════");
    println!();

    for (i, word) in words.iter().enumerate() {
        print!("{:3}. {:<12}", i + 1, word);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════");
    println!();

    let confirmed = prompt_yes_no("Have you saved this Passkey?", true)?;

    if !confirmed {
        return Err(anyhow!("You must save the Passkey to continue").into());
    }

    Ok(words)
}

/// Import an existing Passkey
fn import_passkey() -> Result<Vec<String>> {
    println!("Enter your 24-word Passkey (space-separated):");
    println!("Hint: Press Enter to validate when done");
    println!();

    let input = prompt_input("> ")?;
    let words: Vec<String> = input.split_whitespace().map(String::from).collect();

    if words.len() != 12 && words.len() != 24 {
        return Err(anyhow!(
            "Passkey must be 12 or 24 words (current: {} words)",
            words.len()
        )
        .into());
    }

    // Validate BIP39 checksum
    Passkey::from_words(&words).map_err(|e| anyhow!("Invalid Passkey: {}", e))?;

    println!("✓ Passkey validated successfully");

    Ok(words)
}

/// Prompt for a choice
fn prompt_choice(prompt: &str, options: &[(&str, &str)]) -> Result<String> {
    println!("{}", prompt);
    for (key, desc) in options {
        println!("  [{}] {}", key, desc);
    }
    println!();

    loop {
        let input = prompt_input(&format!(
            "请输入选择 [{}-{}]: ",
            options.first().map(|(k, _)| *k).unwrap_or("1"),
            options.last().map(|(k, _)| *k).unwrap_or("2")
        ))?;

        if options.iter().any(|(k, _)| *k == input) {
            return Ok(input);
        }

        println!("无效的选择，请重试");
    }
}

/// Prompt for yes/no confirmation
fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let default_hint = if default { "[Y/n]" } else { "[y/N]" };

    loop {
        let input = prompt_input(&format!("{} {} ", prompt, default_hint))?.to_lowercase();

        match input.as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please enter y/yes or n/no"),
        }
    }
}

/// Prompt for password (hidden input)
fn prompt_password(prompt: &str) -> Result<String> {
    use std::io::Write;

    print!("{}", prompt);
    std::io::stdout().flush()?;

    // Note: In a real terminal, you'd use rpassword or similar
    // For now, we'll use regular input but note that this should be improved
    prompt_input("")
}

/// Prompt for regular input
fn prompt_input(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;

    // Handle EOF (stdin closed or no input available)
    if bytes_read == 0 {
        return Err(anyhow!("No input available (EOF)").into());
    }

    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_args_parse() {
        use clap::Parser;

        let args = WizardArgs::parse_from(&["wizard"]);
        // Just verify it parses
        let _ = args;
    }
}
