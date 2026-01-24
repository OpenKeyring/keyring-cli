use crate::error::{KeyringError, Result};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub encryption_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub key_derivation: String,
    pub argon2id_params: Argon2idParams,
    pub pbkdf2_iterations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Argon2idParams {
    pub time: u32,
    pub memory: usize,
    pub parallelism: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenKeyringConfig {
    pub database: DatabaseConfig,
    pub crypto: CryptoConfig,
    pub sync: SyncConfig,
    pub clipboard: ClipboardConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub provider: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub conflict_resolution: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub timeout_seconds: u64,
    pub clear_after_copy: bool,
    pub max_content_length: usize,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            clear_after_copy: true,
            max_content_length: 1024,
        }
    }
}

impl Default for OpenKeyringConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: get_default_database_path(),
                encryption_enabled: true,
            },
            crypto: CryptoConfig {
                key_derivation: "argon2id".to_string(),
                argon2id_params: Argon2idParams::default(),
                pbkdf2_iterations: 600_000,
            },
            sync: SyncConfig {
                enabled: false,
                provider: "icloud".to_string(),
                remote_path: "/OpenKeyring".to_string(),
                auto_sync: false,
                conflict_resolution: "newer".to_string(),
            },
            clipboard: ClipboardConfig::default(),
        }
    }
}

impl Default for Argon2idParams {
    fn default() -> Self {
        Self {
            time: 3,
            memory: 64 * 1024 * 1024, // 64MB
            parallelism: 2,
        }
    }
}

pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = get_config_dir();
        let config_file = config_dir.join("config.yaml");

        fs::create_dir_all(&config_dir)?;

        if !config_file.exists() {
            let default_config = OpenKeyringConfig::default();
            save_config(&config_file, &default_config)?;
        }

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    pub fn get_database_config(&self) -> Result<DatabaseConfig> {
        let config = self.load_config()?;
        Ok(config.database)
    }

    pub fn get_crypto_config(&self) -> Result<CryptoConfig> {
        let config = self.load_config()?;
        Ok(config.crypto)
    }

    pub fn get_sync_config(&self) -> Result<SyncConfig> {
        let config = self.load_config()?;
        Ok(config.sync)
    }

    pub fn get_clipboard_config(&self) -> Result<ClipboardConfig> {
        let config = self.load_config()?;
        Ok(config.clipboard)
    }

    pub fn get_keystore_path(&self) -> PathBuf {
        self.config_dir.join("keystore.json")
    }

    pub fn get_master_password(&self) -> Result<String> {
        if let Ok(password) = std::env::var("OK_MASTER_PASSWORD") {
            if !password.is_empty() {
                return Ok(password);
            }
        }

        use rpassword::read_password;
        use std::io::Write;

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

    fn load_config(&self) -> Result<OpenKeyringConfig> {
        let content = fs::read_to_string(&self.config_file)
            .map_err(|e| KeyringError::IoError(e.to_string()))?;
        let config: OpenKeyringConfig = serde_yaml::from_str(&content)
            .map_err(|e| KeyringError::ConfigurationError { context: e.to_string() })?;
        Ok(config)
    }
}

fn get_config_dir() -> PathBuf {
    if let Ok(config_dir) = std::env::var("OK_CONFIG_DIR") {
        PathBuf::from(config_dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_default();
        home_dir.join(".config").join("open-keyring")
    }
}

fn get_default_database_path() -> String {
    if let Ok(data_dir) = std::env::var("OK_DATA_DIR") {
        format!("{}/passwords.db", data_dir)
    } else {
        let home_dir = dirs::home_dir().unwrap_or_default();
        format!("{}/.local/share/open-keyring/passwords.db", home_dir.to_string_lossy())
    }
}

fn save_config(path: &PathBuf, config: &OpenKeyringConfig) -> Result<()> {
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| KeyringError::ConfigurationError { context: e.to_string() })?;
    fs::write(path, yaml)
        .map_err(|e| KeyringError::IoError(e.to_string()))?;
    Ok(())
}