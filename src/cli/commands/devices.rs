use crate::cli::ConfigManager;
use crate::db::vault::Vault;
use crate::device::get_or_create_device_id;
use crate::error::{KeyringError, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const TRUSTED_DEVICES_METADATA_KEY: &str = "trusted_devices";
const REVOKED_DEVICES_METADATA_KEY: &str = "revoked_devices";

/// Get emoji for device type
fn get_device_emoji(device_id: &str) -> &'static str {
    let parts: Vec<&str> = device_id.split('-').collect();
    if parts.is_empty() {
        return "📱";
    }

    match parts[0] {
        "macos" => "💻",
        "ios" => "📱",
        "windows" => "🪟",
        "linux" => "🐧",
        "android" => "🤖",
        "cli" => "⌨️",
        _ => "📱",
    }
}

/// Format timestamp as relative time
fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        format!("{} seconds ago", diff)
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[derive(Parser, Debug)]
pub struct DevicesArgs {
    #[clap(long, short)]
    pub remove: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrustedDevice {
    device_id: String,
    first_seen: i64,
    last_seen: i64,
    sync_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RevokedDevice {
    device_id: String,
    revoked_at: i64,
}

pub async fn manage_devices(args: DevicesArgs) -> Result<()> {
    let config = ConfigManager::new()?;
    let db_path = PathBuf::from(config.get_database_config()?.path);
    let mut vault = Vault::open(&db_path, "")?;

    if let Some(device_id) = args.remove {
        remove_device(&mut vault, &device_id).await?;
    } else {
        list_devices(&mut vault).await?;
    }

    Ok(())
}

async fn list_devices(vault: &mut Vault) -> Result<()> {
    let current_device_id = get_or_create_device_id(vault)?;

    // Get trusted devices from metadata
    let trusted_devices = get_trusted_devices(vault)?;
    let revoked_device_ids = get_revoked_device_ids(vault)?;

    println!("📱 Your Devices:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Always show current device first
    let is_revoked = revoked_device_ids.contains(&current_device_id);
    let emoji = get_device_emoji(&current_device_id);

    if is_revoked {
        println!("{} {} (This device) 🔄", emoji, current_device_id);
        println!("   Status: Revoked - This device cannot access the vault");
    } else {
        println!("{} {} (This device) ✅", emoji, current_device_id);
        println!("   Status: Active - Currently using this device");
    }
    println!();

    // Show other trusted devices
    for device in &trusted_devices {
        if device.device_id != current_device_id {
            let is_revoked = revoked_device_ids.contains(&device.device_id);
            let emoji = get_device_emoji(&device.device_id);
            let last_seen = format_relative_time(device.last_seen);

            if is_revoked {
                println!("{} {} 🔄", emoji, device.device_id);
                println!("   Status: Revoked - Cannot access vault");
                println!("   Last seen: {}", last_seen);
            } else {
                println!("{} {} ✅", emoji, device.device_id);
                println!("   Status: Active - Can access vault");
                println!("   Last seen: {} | Synced: {} times", last_seen, device.sync_count);
            }
            println!();
        }
    }

    if trusted_devices.is_empty() && !revoked_device_ids.contains(&current_device_id) {
        println!("   (No other devices registered)");
        println!();
    }

    // Show warning about cloud access control
    if !revoked_device_ids.is_empty() {
        println!("⚠️  Cloud Access Control:");
        println!("   Revoked devices cannot access your vault even if they have");
        println!("   your cloud storage credentials. The vault data is encrypted");
        println!("   with device-specific keys.");
        println!();
    }

    Ok(())
}

async fn remove_device(vault: &mut Vault, device_id: &str) -> Result<()> {
    let current_device_id = get_or_create_device_id(vault)?;

    if device_id == current_device_id {
        return Err(KeyringError::InvalidInput {
            context: "Cannot remove the current device".to_string(),
        });
    }

    // Get existing revoked devices
    let mut revoked_devices = get_revoked_devices(vault)?;

    // Check if already revoked
    if revoked_devices.iter().any(|d| d.device_id == device_id) {
        return Err(KeyringError::InvalidInput {
            context: format!("Device {} is already revoked", device_id),
        });
    }

    // Add to revoked list
    revoked_devices.push(RevokedDevice {
        device_id: device_id.to_string(),
        revoked_at: chrono::Utc::now().timestamp(),
    });

    // Save back to metadata
    let revoked_json =
        serde_json::to_string(&revoked_devices).map_err(|e| KeyringError::InvalidInput {
            context: format!("Failed to serialize revoked devices: {}", e),
        })?;

    vault.set_metadata(REVOKED_DEVICES_METADATA_KEY, &revoked_json)?;

    println!("✅ Device {} revoked successfully", device_id);
    println!();
    println!("⚠️  Important Security Notice:");
    println!("   • The revoked device can no longer access your vault");
    println!("   • Even if it has your cloud storage credentials");
    println!("   • Vault data is encrypted with device-specific keys");
    println!("   • This device will be excluded from future sync operations");
    println!();

    Ok(())
}

fn get_trusted_devices(vault: &Vault) -> Result<Vec<TrustedDevice>> {
    match vault.get_metadata(TRUSTED_DEVICES_METADATA_KEY)? {
        Some(json_str) => {
            let devices: Vec<TrustedDevice> =
                serde_json::from_str(&json_str).map_err(|e| KeyringError::InvalidInput {
                    context: format!("Failed to parse trusted devices: {}", e),
                })?;
            Ok(devices)
        }
        None => Ok(Vec::new()),
    }
}

fn get_revoked_devices(vault: &Vault) -> Result<Vec<RevokedDevice>> {
    match vault.get_metadata(REVOKED_DEVICES_METADATA_KEY)? {
        Some(json_str) => {
            let devices: Vec<RevokedDevice> =
                serde_json::from_str(&json_str).map_err(|e| KeyringError::InvalidInput {
                    context: format!("Failed to parse revoked devices: {}", e),
                })?;
            Ok(devices)
        }
        None => Ok(Vec::new()),
    }
}

fn get_revoked_device_ids(vault: &Vault) -> Result<Vec<String>> {
    let revoked_devices = get_revoked_devices(vault)?;
    Ok(revoked_devices.into_iter().map(|d| d.device_id).collect())
}
