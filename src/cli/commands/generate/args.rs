//! Command arguments for password generation
//!
//! Defines the NewArgs struct and PasswordType enum.

use crate::error::{KeyringError, Result};
use clap::Parser;

/// Arguments for the generate command (now accessible via 'new' subcommand)
#[derive(Parser, Debug)]
pub struct NewArgs {
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

impl NewArgs {
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
                        context: "Memorable password word count must be between 3 and 12"
                            .to_string(),
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

/// Get description for password type
pub fn get_password_description(password_type: PasswordType) -> String {
    match password_type {
        PasswordType::Random => "Random password with special characters".to_string(),
        PasswordType::Memorable => "Memorable word-based passphrase".to_string(),
        PasswordType::Pin => "Numeric PIN code".to_string(),
    }
}
