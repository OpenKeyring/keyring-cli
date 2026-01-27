//! Device identification utilities

use crate::db::vault::Vault;
use crate::error::Result;
use rand::Rng;

const DEVICE_ID_METADATA_KEY: &str = "device_id";

pub fn generate_device_id() -> String {
    let platform = std::env::consts::OS;
    let device_name = sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string());
    let normalized_device_name = device_name.replace('-', "_");
    let fingerprint = generate_fingerprint();

    format!("{}-{}-{}", platform, normalized_device_name, fingerprint)
}

pub fn get_or_create_device_id(vault: &mut Vault) -> Result<String> {
    if let Some(existing) = vault.get_metadata(DEVICE_ID_METADATA_KEY)? {
        return Ok(existing);
    }

    let device_id = generate_device_id();
    vault.set_metadata(DEVICE_ID_METADATA_KEY, &device_id)?;

    Ok(device_id)
}

fn generate_fingerprint() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 4] = rng.random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
