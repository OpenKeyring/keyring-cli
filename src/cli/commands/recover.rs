//! Recover vault using Passkey
//!
//! This command allows users to recover their vault by providing their 24-word Passkey
//! and setting a new master password. The Passkey is used to derive the root master key,
//! which is then used to re-encrypt the wrapped_passkey with the new device password.

use crate::cli::ConfigManager;
use crate::crypto::{passkey::Passkey, CryptoManager};
use crate::error::{KeyringError, Result};
use crate::db::vault::Vault;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

use base64::Engine;

#[derive(Parser, Debug)]
pub struct RecoverArgs {
    /// 24-word Passkey (optional, will prompt if not provided)
    #[arg(long, short)]
    pub passkey: Option<String>,
}

pub async fn execute(args: RecoverArgs) -> Result<()> {
    println!("🔐 Recovery Mode");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Get Passkey from argument or prompt
    let passkey_words = if let Some(passkey_str) = args.passkey {
        println!("✓ Passkey provided via argument");
        parse_passkey_input(&passkey_str)?
    } else {
        prompt_for_passkey()?
    };

    // Validate Passkey
    let passkey = Passkey::from_words(&passkey_words).map_err(|e| KeyringError::InvalidInput {
        context: format!("Invalid Passkey: {}", e),
    })?;

    println!("✓ Passkey validated successfully");
    println!();

    // Prompt for new password
    let new_password = prompt_for_new_password()?;

    // Initialize CryptoManager with Passkey
    let mut crypto = CryptoManager::new();

    // Derive root master key from Passkey
    let seed = passkey.to_seed(None).map_err(|e| KeyringError::Crypto {
        context: format!("Failed to derive Passkey seed: {}", e),
    })?;

    // Generate new salt for recovery
    let salt = crate::crypto::argon2id::generate_salt();
    let root_master_key = seed.derive_root_master_key(&salt).map_err(|e| KeyringError::Crypto {
        context: format!("Failed to derive root master key: {}", e),
    })?;

    // Generate KDF nonce for device key derivation
    let kdf_nonce = generate_kdf_nonce();

    // Initialize with Passkey (using CLI device index)
    use crate::crypto::hkdf::DeviceIndex;
    crypto
        .initialize_with_passkey(
            &passkey,
            &new_password,
            &root_master_key,
            DeviceIndex::CLI,
            &kdf_nonce,
        )
        .map_err(|e| KeyringError::Crypto {
            context: format!("Failed to initialize with Passkey: {}", e),
        })?;

    println!("✓ Vault recovered successfully");
    println!();
    println!("⚠️  Important Notes:");
    println!("   • Your vault has been re-encrypted with the new password");
    println!("   • The old password will no longer work");
    println!("   • Keep your Passkey safe - it's required for future recoveries");
    println!("   • Each device has its own independent password");
    println!();

    // Store salt and KDF nonce in vault metadata for future reference
    let config = ConfigManager::new()?;
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);
    let mut vault = Vault::open(&db_path, "")?;

    // Store salt as base64 for persistence
    let salt_b64 = base64::engine::general_purpose::STANDARD.encode(salt);
    vault.set_metadata("recovery_salt", &salt_b64)?;

    let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(kdf_nonce);
    vault.set_metadata("recovery_kdf_nonce", &nonce_b64)?;

    println!("✓ Recovery metadata saved");

    Ok(())
}

/// Parse Passkey input from string (space or comma-separated)
fn parse_passkey_input(input: &str) -> Result<Vec<String>> {
    let words: Vec<String> = input
        .split(&[',', ' '][..])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if words.is_empty() {
        return Err(KeyringError::InvalidInput {
            context: "Passkey cannot be empty".to_string(),
        });
    }

    Ok(words)
}

/// Prompt user for 24-word Passkey
fn prompt_for_passkey() -> Result<Vec<String>> {
    println!("Enter your 24-word Passkey (space-separated):");
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let words = parse_passkey_input(&input)?;

    if words.len() != 24 {
        return Err(KeyringError::InvalidInput {
            context: format!(
                "Passkey must be exactly 24 words, got {} words",
                words.len()
            ),
        });
    }

    // Validate each word is a valid BIP39 word
    for (i, word) in words.iter().enumerate() {
        if !Passkey::is_valid_word(word) {
            return Err(KeyringError::InvalidInput {
                context: format!("Invalid BIP39 word at position {}: '{}'", i + 1, word),
            });
        }
    }

    Ok(words)
}

/// Prompt user for new password with confirmation
fn prompt_for_new_password() -> Result<String> {
    println!("Set a new master password for this device:");
    println!("(Minimum 8 characters, recommended: 16+ with mixed characters)");
    println!();

    // Prompt for password
    print!("New password: ");
    io::stdout().flush()?;
    let new_password = rpassword::read_password()?;

    if new_password.len() < 8 {
        return Err(KeyringError::InvalidInput {
            context: "Password must be at least 8 characters".to_string(),
        });
    }

    // Confirm password
    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm_password = rpassword::read_password()?;

    if new_password != confirm_password {
        return Err(KeyringError::InvalidInput {
            context: "Passwords do not match".to_string(),
        });
    }

    Ok(new_password)
}

/// Generate a random KDF nonce for device key derivation
fn generate_kdf_nonce() -> [u8; 32] {
    use rand::Rng;
    let mut nonce = [0u8; 32];
    let mut rng = rand::rng();
    rng.fill(&mut nonce);
    nonce
}
