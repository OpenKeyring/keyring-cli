//! Generate password command
//!
//! This module provides password generation functionality with three types:
//! - Random: High-entropy random passwords with special characters
//! - Memorable: Word-based passphrases (e.g., "correct-horse-battery-staple")
//! - PIN: Numeric PIN codes

use clap::Parser;
use crate::cli::ConfigManager;
use crate::crypto::CryptoManager;
use crate::error::{KeyringError, Result};
use crate::db::vault::Vault;
use crate::db::models::{Record, RecordType};
use crate::clipboard::{ClipboardService, create_platform_clipboard, ClipboardConfig};
use std::path::PathBuf;

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

    /// Type of password to generate: random, memorable, or pin
    #[clap(short, long, default_value = "random")]
    pub r#type: String,

    /// Tags for categorizing the password
    #[clap(long, short)]
    pub tags: Vec<String>,

    /// Sync to cloud after generating
    #[clap(long)]
    pub sync: bool,

    /// Number of words for memorable passwords (default: 4)
    #[clap(long, default_value = "4")]
    pub words: usize,

    /// Do not copy to clipboard
    #[clap(long)]
    pub no_clipboard: bool,
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

    /// Determine password type from arguments
    pub fn get_password_type(&self) -> Result<PasswordType> {
        // Priority: flags > type parameter
        if self.pin {
            return Ok(PasswordType::Pin);
        }
        if self.memorable {
            return Ok(PasswordType::Memorable);
        }

        // Parse type parameter
        match self.r#type.to_lowercase().as_str() {
            "random" => Ok(PasswordType::Random),
            "memorable" => Ok(PasswordType::Memorable),
            "pin" => Ok(PasswordType::Pin),
            _ => Err(KeyringError::InvalidInput {
                context: format!("Invalid password type: {}. Must be: random, memorable, or pin", self.r#type),
            }),
        }
    }
}

/// Password generation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordType {
    Random,
    Memorable,
    Pin,
}

/// Execute the generate command
pub fn execute(args: GenerateArgs) -> Result<()> {
    // Validate arguments
    args.validate()?;

    // Initialize configuration
    let config = ConfigManager::new()?;

    // Initialize crypto manager with master password
    let master_password = prompt_for_master_password()?;
    let mut crypto = CryptoManager::new();
    crypto.initialize(&master_password)?;

    // Generate password based on type
    let password_type = args.get_password_type()?;
    let password = match password_type {
        PasswordType::Random => crypto.generate_random_password(args.length)?,
        PasswordType::Memorable => crypto.generate_memorable_password(args.words)?,
        PasswordType::Pin => crypto.generate_pin(args.length)?,
    };

    // Encrypt the password
    let encrypted_data = crypto.encrypt(password.as_bytes())?;
    let encrypted_string = base64::encode(&encrypted_data.0);

    // Create record
    let record = Record {
        id: uuid::Uuid::new_v4(),
        record_type: RecordType::Password,
        encrypted_data: encrypted_string,
        name: args.name.clone(),
        username: None,
        url: None,
        notes: Some(get_password_description(password_type)),
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

    // Copy to clipboard
    if !args.no_clipboard {
        copy_to_clipboard(&password)?;
    }

    // Print success message
    print_success_message(&args.name, &password, password_type);

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
    let clipboard_config = ClipboardConfig::default();
    let mut clipboard = ClipboardService::new(clipboard_manager, clipboard_config);

    clipboard.copy_password(password)?;
    Ok(())
}

/// Print success message with password details
fn print_success_message(name: &str, password: &str, password_type: PasswordType) {
    println!("✅ Password generated successfully!");
    println!("   Name: {}", name);
    println!("   Type: {}", format!("{:?}", password_type).to_lowercase());
    println!("   Length: {}", password.len());

    // Show password (in production, this should be optional)
    println!("   Password: {}", password);

    // Clipboard notice
    println!("   📋 Copied to clipboard (auto-clears in 30s)");
}

// Re-export for backward compatibility
pub use execute as generate_password;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_args_validation_empty_name() {
        let args = GenerateArgs {
            name: String::new(),
            length: 16,
            memorable: false,
            pin: false,
            r#type: "random".to_string(),
            tags: vec![],
            sync: false,
            words: 4,
            no_clipboard: true,
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_generate_args_validation_invalid_length() {
        let args = GenerateArgs {
            name: "test".to_string(),
            length: 2, // Too short
            memorable: false,
            pin: false,
            r#type: "random".to_string(),
            tags: vec![],
            sync: false,
            words: 4,
            no_clipboard: true,
        };

        assert!(args.validate().is_err());
    }

    #[test]
    fn test_generate_args_get_password_type() {
        let args = GenerateArgs {
            name: "test".to_string(),
            length: 16,
            memorable: false,
            pin: false,
            r#type: "random".to_string(),
            tags: vec![],
            sync: false,
            words: 4,
            no_clipboard: true,
        };

        assert_eq!(args.get_password_type().unwrap(), PasswordType::Random);
    }

    #[test]
    fn test_generate_args_pin_flag() {
        let args = GenerateArgs {
            name: "test".to_string(),
            length: 6,
            memorable: false,
            pin: true, // PIN flag takes priority
            r#type: "random".to_string(),
            tags: vec![],
            sync: false,
            words: 4,
            no_clipboard: true,
        };

        assert_eq!(args.get_password_type().unwrap(), PasswordType::Pin);
    }

    #[test]
    fn test_generate_args_invalid_type() {
        let args = GenerateArgs {
            name: "test".to_string(),
            length: 16,
            memorable: false,
            pin: false,
            r#type: "invalid".to_string(),
            tags: vec![],
            sync: false,
            words: 4,
            no_clipboard: true,
        };

        assert!(args.get_password_type().is_err());
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
}