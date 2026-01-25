//! Generate password command
//!
//! This module provides password generation functionality with three types:
//! - Random: High-entropy random passwords with special characters
//! - Memorable: Word-based passphrases (e.g., "Correct-Horse-Battery-Staple")
//! - PIN: Numeric PIN codes

use clap::Parser;
use crate::cli::ConfigManager;
use crate::crypto::{CryptoManager, keystore::KeyStore, record::{RecordPayload, encrypt_payload}};
use crate::error::{KeyringError, Result};
use crate::db::vault::Vault;
use crate::db::models::{RecordType, StoredRecord};
use crate::clipboard::{ClipboardService, ClipboardConfig, create_platform_clipboard};
use crate::onboarding::is_initialized;
use std::io::Write;
use std::path::PathBuf;
use rand::Rng;
use rand::seq::SliceRandom;

/// Arguments for the generate command
#[derive(Parser, Debug)]
pub struct GenerateArgs {
    /// Name/identifier for the password
    #[clap(short, long)]
    pub name: String,

    /// Length of password (for random/pin types)
    #[clap(short, long, default_value = "16")]
    pub length: usize,

    /// Generate memorable password (word-based passphrase)
    #[clap(short, long)]
    pub memorable: bool,

    /// Generate numeric PIN
    #[clap(short, long)]
    pub pin: bool,

    /// Include numbers in random password
    #[clap(long, default_value = "true")]
    pub numbers: bool,

    /// Include symbols in random password
    #[clap(long, default_value = "true")]
    pub symbols: bool,

    /// Username for the password record
    #[clap(short, long)]
    pub username: Option<String>,

    /// URL for the password record
    #[clap(long)]
    pub url: Option<String>,

    /// Notes for the password record
    #[clap(long)]
    pub notes: Option<String>,

    /// Tags for categorizing the password
    #[clap(long, short, value_delimiter = ',')]
    pub tags: Vec<String>,

    /// Sync to cloud after generating
    #[clap(long)]
    pub sync: bool,

    /// Number of words for memorable passwords (default: 4)
    #[clap(long, default_value = "4")]
    pub words: usize,

    /// Copy to clipboard
    #[clap(short, long)]
    pub copy: bool,
}

impl GenerateArgs {
    /// Validate the generate arguments
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(KeyringError::InvalidInput {
                context: "Name cannot be empty".to_string(),
            });
        }

        // Validate length based on type
        let password_type = self.get_password_type()?;
        match password_type {
            PasswordType::Random => {
                if self.length < 4 || self.length > 128 {
                    return Err(KeyringError::InvalidInput {
                        context: "Random password length must be between 4 and 128".to_string(),
                    });
                }
            }
            PasswordType::Memorable => {
                if self.words < 3 || self.words > 12 {
                    return Err(KeyringError::InvalidInput {
                        context: "Memorable password word count must be between 3 and 12".to_string(),
                    });
                }
            }
            PasswordType::Pin => {
                if self.length < 4 || self.length > 16 {
                    return Err(KeyringError::InvalidInput {
                        context: "PIN length must be between 4 and 16".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Determine password type from arguments (using boolean flags only)
    pub fn get_password_type(&self) -> Result<PasswordType> {
        // Priority: pin > memorable > random
        if self.pin {
            return Ok(PasswordType::Pin);
        }
        if self.memorable {
            return Ok(PasswordType::Memorable);
        }
        Ok(PasswordType::Random)
    }
}

/// Password generation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordType {
    Random,
    Memorable,
    Pin,
}

/// Generate a random password with specified length and character set
///
/// # Arguments
/// * `length` - The desired password length
/// * `numbers` - Whether to include numeric characters
/// * `symbols` - Whether to include special symbols
///
/// # Returns
/// A randomly generated password string
///
/// # Character Sets
/// - Lowercase: "abcdefghijkmnpqrstuvwxyz" (excludes: o, l)
/// - Uppercase: "ABCDEFGHJKLMNPQRSTUVWXYZ" (excludes: O, I)
/// - Numbers: "23456789" (excludes: 0, 1)
/// - Symbols: "!@#$%^&*()_+-=[]{}|;:,.<>?"
pub fn generate_random(length: usize, numbers: bool, symbols: bool) -> Result<String> {
    if length < 4 {
        return Err(KeyringError::InvalidInput {
            context: "Password length must be at least 4 characters".to_string(),
        });
    }
    if length > 128 {
        return Err(KeyringError::InvalidInput {
            context: "Password length cannot exceed 128 characters".to_string(),
        });
    }

    // Character sets excluding ambiguous characters
    let lowercase = "abcdefghijkmnpqrstuvwxyz"; // no o, l
    let uppercase = "ABCDEFGHJKLMNPQRSTUVWXYZ"; // no O, I
    let nums = "23456789"; // no 0, 1
    let syms = "!@#$%^&*()_+-=[]{}|;:,.<>?";

    let mut charset = String::from(lowercase);
    charset.push_str(uppercase);

    if numbers {
        charset.push_str(nums);
    }
    if symbols {
        charset.push_str(syms);
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx]
        })
        .collect();

    Ok(password)
}

/// Generate a memorable password using word-based approach
///
/// # Arguments
/// * `word_count` - The number of words to include (3-12)
///
/// # Returns
/// A memorable passphrase with capitalized words (e.g., "Correct-Horse-Battery-Staple")
pub fn generate_memorable(word_count: usize) -> Result<String> {
    const WORDS: &[&str] = &[
        "correct", "horse", "battery", "staple", "apple", "banana", "cherry", "dragon",
        "elephant", "flower", "garden", "house", "island", "jungle", "kangaroo", "lemon",
        "mountain", "nectar", "orange", "piano", "queen", "river", "sunshine", "tiger",
        "umbrella", "violet", "whale", "xylophone", "yellow", "zebra", "castle", "desert",
        "eagle", "forest", "giraffe", "harbor", "igloo", "journey", "kingdom", "lantern",
        "meadow", "night", "ocean", "planet", "quartz", "rainbow", "star", "tower",
        "universe", "valley", "wave", "crystal", "year", "zen", "bridge", "cloud",
        "diamond", "emerald", "fountain", "galaxy", "horizon", "infinity", "jewel",
    ];

    if word_count < 3 {
        return Err(KeyringError::InvalidInput {
            context: "Word count must be at least 3".to_string(),
        });
    }
    if word_count > 12 {
        return Err(KeyringError::InvalidInput {
            context: "Word count cannot exceed 12".to_string(),
        });
    }

    let mut rng = rand::thread_rng();
    let selected: Vec<&str> = WORDS.choose_multiple(&mut rng, word_count)
        .map(|w| *w)
        .collect();

    // Capitalize first letter of each word and join with hyphens
    let password = selected.iter()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    Ok(password)
}

/// Generate a numeric PIN
///
/// # Arguments
/// * `length` - The desired PIN length (4-16)
///
/// # Returns
/// A numeric PIN using digits 2-9 only (excludes 0 and 1 for clarity)
pub fn generate_pin(length: usize) -> Result<String> {
    if length < 4 {
        return Err(KeyringError::InvalidInput {
            context: "PIN length must be at least 4 digits".to_string(),
        });
    }
    if length > 16 {
        return Err(KeyringError::InvalidInput {
            context: "PIN length cannot exceed 16 digits".to_string(),
        });
    }

    // Use only 2-9 to avoid ambiguous 0 and 1
    let digits = [b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
    let mut rng = rand::thread_rng();
    let pin: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..digits.len());
            digits[idx] as char
        })
        .collect();

    Ok(pin)
}

/// Execute the generate command
pub async fn execute(args: GenerateArgs) -> Result<()> {
    // Validate arguments
    args.validate()?;

    // Initialize configuration
    let config = ConfigManager::new()?;

    // Initialize keystore and crypto manager with DEK
    let master_password = config.get_master_password()?;
    let keystore_path = config.get_keystore_path();
    let keystore = if is_initialized(&keystore_path) {
        KeyStore::unlock(&keystore_path, &master_password)?
    } else {
        let keystore = KeyStore::initialize(&keystore_path, &master_password)?;
        if let Some(recovery_key) = &keystore.recovery_key {
            println!("🔑 Recovery Key (save securely): {}", recovery_key);
        }
        keystore
    };
    let mut crypto = CryptoManager::new();
    crypto.initialize_with_key(keystore.dek);

    // Generate password based on type
    let password_type = args.get_password_type()?;
    let password = match password_type {
        PasswordType::Random => generate_random(args.length, args.numbers, args.symbols)?,
        PasswordType::Memorable => generate_memorable(args.words)?,
        PasswordType::Pin => generate_pin(args.length)?,
    };

    let payload = RecordPayload {
        name: args.name.clone(),
        username: args.username.clone(),
        password: password.clone(),
        url: args.url.clone(),
        notes: args
            .notes
            .clone()
            .or(Some(get_password_description(password_type))),
        tags: args.tags.clone(),
    };
    let (encrypted_data, nonce) = encrypt_payload(&crypto, &payload)?;

    // Create record
    let record = StoredRecord {
        id: uuid::Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data,
        nonce,
        tags: args.tags.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Get database path
    let db_config = config.get_database_config()?;
    let db_path = PathBuf::from(db_config.path);

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save to database
    let mut vault = Vault::open(&db_path, &master_password)?;
    vault.add_record(&record)?;

    // Copy to clipboard (only if --copy flag is set)
    if args.copy {
        copy_to_clipboard(&password)?;
    }

    // Print success message
    print_success_message(&args.name, &password, password_type, args.copy);

    // Handle sync if requested
    if args.sync {
        println!("🔄 Sync to cloud requested (not yet implemented)");
    }

    Ok(())
}

/// Prompt user for master password
fn prompt_for_master_password() -> Result<String> {
    use rpassword::read_password;

    print!("🔐 Enter master password: ");
    let _ = std::io::stdout().flush();
    let password = read_password()
        .map_err(|e| KeyringError::IoError(format!("Failed to read password: {}", e)))?;

    if password.is_empty() {
        return Err(KeyringError::AuthenticationFailed {
            reason: "Master password cannot be empty".to_string(),
        });
    }

    Ok(password)
}

/// Get description for password type
fn get_password_description(password_type: PasswordType) -> String {
    match password_type {
        PasswordType::Random => "Random password with special characters".to_string(),
        PasswordType::Memorable => "Memorable word-based passphrase".to_string(),
        PasswordType::Pin => "Numeric PIN code".to_string(),
    }
}

/// Copy password to clipboard securely
fn copy_to_clipboard(password: &str) -> Result<()> {
    let clipboard_manager = create_platform_clipboard()?;
    let mut clipboard = ClipboardService::new(clipboard_manager, ClipboardConfig::default());

    clipboard.copy_password(password)?;
    Ok(())
}

/// Print success message with password details
fn print_success_message(name: &str, password: &str, password_type: PasswordType, copied: bool) {
    println!("✅ Password generated successfully!");
    println!("   Name: {}", name);
    println!("   Type: {}", format!("{:?}", password_type).to_lowercase());
    println!("   Length: {}", password.len());

    // Show password (in production, this should be optional)
    println!("   Password: {}", password);

    // Clipboard notice (only if copied)
    if copied {
        println!("   📋 Copied to clipboard (auto-clears in 30s)");
    }
}

// Re-export for backward compatibility
pub use execute as generate_password;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args() -> GenerateArgs {
        GenerateArgs {
            name: "test".to_string(),
            length: 16,
            memorable: false,
            pin: false,
            numbers: false,
            symbols: false,
            username: None,
            url: None,
            notes: None,
            tags: vec![],
            sync: false,
            words: 4,
            copy: false,
        }
    }

    #[test]
    fn test_generate_args_validation_empty_name() {
        let mut args = create_test_args();
        args.name = String::new();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_generate_args_validation_invalid_length() {
        let mut args = create_test_args();
        args.length = 2; // Too short
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_generate_args_get_password_type() {
        let args = create_test_args();
        assert_eq!(args.get_password_type().unwrap(), PasswordType::Random);
    }

    #[test]
    fn test_generate_args_memorable_flag() {
        let mut args = create_test_args();
        args.memorable = true;
        assert_eq!(args.get_password_type().unwrap(), PasswordType::Memorable);
    }

    #[test]
    fn test_generate_args_pin_flag() {
        let mut args = create_test_args();
        args.pin = true;
        assert_eq!(args.get_password_type().unwrap(), PasswordType::Pin);
    }

    #[test]
    fn test_generate_args_pin_priority_over_memorable() {
        let mut args = create_test_args();
        args.pin = true;
        args.memorable = true;
        // PIN should take priority
        assert_eq!(args.get_password_type().unwrap(), PasswordType::Pin);
    }

    #[test]
    fn test_password_description() {
        assert_eq!(
            get_password_description(PasswordType::Random),
            "Random password with special characters"
        );
        assert_eq!(
            get_password_description(PasswordType::Memorable),
            "Memorable word-based passphrase"
        );
        assert_eq!(
            get_password_description(PasswordType::Pin),
            "Numeric PIN code"
        );
    }

    #[test]
    fn test_generate_random_password_length() {
        let password = generate_random(16, true, true).unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_generate_random_password_no_ambiguous_chars() {
        let password = generate_random(100, true, true).unwrap();
        // Should not contain ambiguous characters
        assert!(!password.contains('0'));
        assert!(!password.contains('1'));
        assert!(!password.contains('O'));
        assert!(!password.contains('I'));
        assert!(!password.contains('o'));
        assert!(!password.contains('l'));
    }

    #[test]
    fn test_generate_random_password_without_numbers_or_symbols() {
        let password = generate_random(16, false, false).unwrap();
        // Should only contain letters (no numbers or symbols)
        assert!(password.chars().all(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_generate_random_password_too_short() {
        let result = generate_random(3, true, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_random_password_too_long() {
        let result = generate_random(129, true, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_memorable_password_word_count() {
        let password = generate_memorable(4).unwrap();
        // Should have 4 words separated by hyphens
        let parts: Vec<&str> = password.split('-').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn test_generate_memorable_password_capitalization() {
        let password = generate_memorable(4).unwrap();
        // Each word should start with uppercase
        for word in password.split('-') {
            assert!(word.chars().next().unwrap().is_uppercase());
        }
    }

    #[test]
    fn test_generate_memorable_password_too_few_words() {
        let result = generate_memorable(2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_memorable_password_too_many_words() {
        let result = generate_memorable(13);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_pin_length() {
        let pin = generate_pin(6).unwrap();
        assert_eq!(pin.len(), 6);
    }

    #[test]
    fn test_generate_pin_only_2_to_9() {
        let pin = generate_pin(20).unwrap();
        // Should only contain digits 2-9
        assert!(pin.chars().all(|c| c.is_ascii_digit() && c >= '2' && c <= '9'));
        // Should not contain 0 or 1
        assert!(!pin.contains('0'));
        assert!(!pin.contains('1'));
    }

    #[test]
    fn test_generate_pin_too_short() {
        let result = generate_pin(3);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_pin_too_long() {
        let result = generate_pin(17);
        assert!(result.is_err());
    }
}