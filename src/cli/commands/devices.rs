use crate::cli::ConfigManager;
use crate::db::vault::Vault;
use crate::device::get_or_create_device_id;
use crate::error::{KeyringError, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const TRUSTED_DEVICES_METADATA_KEY: &str = "trusted_devices";
const REVOKED_DEVICES_METADATA_KEY: &str = "revoked_devices";

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

    // Always show current device first
    let is_revoked = revoked_device_ids.contains(&current_device_id);
    let status = if is_revoked {
        " (Revoked)"
    } else {
        " (This device)"
    };
    println!("   • {}{}", current_device_id, status);

    // Show other trusted devices
    for device in &trusted_devices {
        if device.device_id != current_device_id {
            let is_revoked = revoked_device_ids.contains(&device.device_id);
            let status = if is_revoked { " (Revoked)" } else { "" };
            let last_seen = chrono::DateTime::from_timestamp(device.last_seen, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            println!(
                "   • {}{} (last seen: {})",
                device.device_id, status, last_seen
            );
        }
    }

    if trusted_devices.is_empty() && !revoked_device_ids.contains(&current_device_id) {
        println!("   (No other devices registered)");
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
