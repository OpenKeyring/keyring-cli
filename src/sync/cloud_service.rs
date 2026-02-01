//! Cloud Sync Service
//!
//! Provides cloud synchronization using OpenDAL-based storage.

use crate::cloud::{
    metadata::{CloudMetadata, DeviceInfo},
    CloudConfig, CloudStorage,
};
use anyhow::Result;
use base64::prelude::*;
use std::collections::HashMap;

/// Cloud sync service for cross-device synchronization
pub struct CloudSyncService {
    /// Cloud storage client
    pub storage: CloudStorage,
    /// KDF nonce for key derivation
    pub kdf_nonce: [u8; 32],
    /// Device identifier
    pub device_id: String,
}

/// Sync direction for synchronization operations
pub enum SyncDirection {
    /// Upload local changes to cloud
    Upload,
    /// Download changes from cloud to local
    Download,
    /// Bidirectional synchronization
    Both,
}

/// Statistics from a sync operation
pub struct SyncStats {
    pub uploaded: usize,
    pub downloaded: usize,
    pub conflicts: usize,
}

impl CloudSyncService {
    /// Create a new cloud sync service
    ///
    /// # Arguments
    ///
    /// * `config` - Cloud provider configuration
    /// * `kdf_nonce` - 32-byte nonce for key derivation
    ///
    /// # Returns
    ///
    /// Returns a `CloudSyncService` instance or an error if configuration is invalid
    pub fn new(config: &CloudConfig, kdf_nonce: &[u8; 32]) -> Result<Self> {
        let mut nonce_array = [0u8; 32];
        nonce_array.copy_from_slice(kdf_nonce);

        let storage = CloudStorage::new(config)?;
        let device_id = Self::generate_device_id()?;

        Ok(Self {
            storage,
            kdf_nonce: nonce_array,
            device_id,
        })
    }

    /// Initialize cloud metadata if it doesn't exist
    ///
    /// Creates a new metadata file with the current device and KDF nonce.
    /// If metadata already exists, this is a no-op.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error if metadata creation fails
    pub async fn initialize_metadata(&self) -> Result<()> {
        if self.storage.metadata_exists().await? {
            return Ok(());
        }

        let device_info = DeviceInfo {
            device_id: self.device_id.clone(),
            platform: Self::get_platform(),
            device_name: Self::get_device_name(),
            last_seen: chrono::Utc::now(),
            sync_count: 0,
        };

        let metadata = CloudMetadata {
            format_version: "1.0".to_string(),
            kdf_nonce: BASE64_STANDARD.encode(self.kdf_nonce),
            created_at: chrono::Utc::now(),
            updated_at: Some(chrono::Utc::now()),
            metadata_version: 1,
            devices: vec![device_info],
            records: HashMap::new(),
        };

        self.storage.upload_metadata(&metadata).await?;
        Ok(())
    }

    /// Perform synchronization in the specified direction
    ///
    /// # Arguments
    ///
    /// * `direction` - Sync direction (Upload, Download, or Both)
    ///
    /// # Returns
    ///
    /// Returns sync statistics or an error if sync fails
    pub async fn sync(&self, direction: SyncDirection) -> Result<SyncStats> {
        match direction {
            SyncDirection::Upload => self.upload().await,
            SyncDirection::Download => self.download().await,
            SyncDirection::Both => {
                let up = self.upload().await?;
                let down = self.download().await?;
                Ok(SyncStats {
                    uploaded: up.uploaded + down.uploaded,
                    downloaded: up.downloaded + down.downloaded,
                    conflicts: up.conflicts + down.conflicts,
                })
            }
        }
    }

    /// Upload local records to cloud storage
    ///
    /// This method uploads records from the local database to cloud storage.
    /// Each record is uploaded as a separate JSON file with format: `{id}-{device_id}.json`
    ///
    /// # Implementation Note
    /// This is a placeholder implementation that demonstrates the upload flow.
    /// Full integration requires access to the vault/database to retrieve records.
    ///
    /// # Returns
    ///
    /// Returns sync statistics with upload count
    async fn upload(&self) -> Result<SyncStats> {
        // Placeholder implementation demonstrating the upload flow
        // In production, this would:
        // 1. Access vault through sync service
        // 2. Get records from local database
        // 3. For each record:
        //    - Create SyncRecord with version
        //    - Upload to cloud storage via storage.upload_record()
        //    - Update metadata

        // For now, list existing cloud files to demonstrate storage access
        let existing_files = self.storage.list_records().await.unwrap_or_default();
        let uploaded = existing_files.len();

        Ok(SyncStats {
            uploaded,
            downloaded: 0,
            conflicts: 0,
        })
    }

    /// Download records from cloud storage
    ///
    /// This method downloads records from cloud storage and merges them with local data.
    /// Files are parsed to extract record ID and device ID from the filename.
    ///
    /// # Implementation Note
    /// This is a placeholder implementation that demonstrates the download flow.
    /// Full integration requires:
    /// - Access to vault/database for local records
    /// - Version comparison logic
    /// - Conflict resolution integration
    ///
    /// # Returns
    ///
    /// Returns sync statistics with download count
    async fn download(&self) -> Result<SyncStats> {
        // List remote files from cloud storage
        let files = self.storage.list_records().await.unwrap_or_default();
        let mut downloaded = 0;

        for filename in files {
            // Parse filename to get record ID and device
            // Format: {id}-{device_id}.json
            if let Some(stripped) = filename.strip_suffix(".json") {
                let parts: Vec<&str> = stripped.splitn(2, '-').collect();
                if parts.len() >= 2 {
                    let _record_id = parts[0]; // Will be used for version comparison
                    let device_id = parts[1];

                    // Check if this record is from our device or another
                    if device_id != self.device_id {
                        // In production:
                        // - Check if local record exists
                        // - Compare versions
                        // - Download if remote version is newer
                        downloaded += 1;
                    }
                }
            }
        }

        Ok(SyncStats {
            uploaded: 0,
            downloaded,
            conflicts: 0,
        })
    }

    /// Generate a unique device identifier
    ///
    /// Format: `{platform}-local-{fingerprint}`
    fn generate_device_id() -> Result<String> {
        let platform = Self::get_platform();

        // Generate 4-byte random fingerprint
        let fingerprint: String = (0..4)
            .map(|_| rand::random::<u8>())
            .map(|b| format!("{:02x}", b))
            .collect();

        Ok(format!("{}-local-{}", platform, fingerprint))
    }

    /// Get the current platform identifier
    fn get_platform() -> String {
        if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "ios") {
            "ios".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "cli".to_string()
        }
    }

    /// Get the device name from the system
    ///
    /// This method attempts to get the actual hostname from the system.
    /// Falls back to platform-specific generic names if hostname is unavailable.
    fn get_device_name() -> String {
        // Try environment variables first
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            return hostname;
        }

        if let Ok(computername) = std::env::var("COMPUTERNAME") {
            return computername;
        }

        // Try to get hostname via sysinfo
        if let Some(host) = sysinfo::System::host_name() {
            if !host.is_empty() {
                return host;
            }
        }

        // Fallback to platform-specific name
        let platform = Self::get_platform();
        match platform.as_str() {
            "macos" => "Mac".to_string(),
            "linux" => "Linux Device".to_string(),
            "windows" => "Windows PC".to_string(),
            _ => format!("{} Device", platform),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud::config::CloudProvider;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cloud_sync_service_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]);
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.kdf_nonce, [1u8; 32]);
        assert!(!service.device_id.is_empty());
    }

    #[tokio::test]
    async fn test_initialize_metadata_creates_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();

        // Metadata should not exist initially
        assert!(!service.storage.metadata_exists().await.unwrap());

        // Initialize should create metadata
        service.initialize_metadata().await.unwrap();

        // Metadata should now exist
        assert!(service.storage.metadata_exists().await.unwrap());
    }

    #[tokio::test]
    async fn test_initialize_metadata_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();

        // First call should create metadata
        service.initialize_metadata().await.unwrap();
        let metadata1 = service.storage.download_metadata().await.unwrap();

        // Second call should be no-op
        service.initialize_metadata().await.unwrap();
        let metadata2 = service.storage.download_metadata().await.unwrap();

        // Metadata should be unchanged
        assert_eq!(metadata1.metadata_version, metadata2.metadata_version);
    }

    #[tokio::test]
    async fn test_sync_upload() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();
        service.initialize_metadata().await.unwrap();

        let stats = service.sync(SyncDirection::Upload).await.unwrap();
        // Should not error, but stats are empty until upload logic is implemented
        assert_eq!(stats.uploaded, 0);
    }

    #[tokio::test]
    async fn test_sync_download() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();
        service.initialize_metadata().await.unwrap();

        let stats = service.sync(SyncDirection::Download).await.unwrap();
        // Should not error, but stats are empty until download logic is implemented
        assert_eq!(stats.downloaded, 0);
    }

    #[tokio::test]
    async fn test_sync_both() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();
        service.initialize_metadata().await.unwrap();

        let stats = service.sync(SyncDirection::Both).await.unwrap();
        // Should not error, but stats are empty until logic is implemented
        assert_eq!(stats.uploaded, 0);
        assert_eq!(stats.downloaded, 0);
    }

    #[test]
    fn test_generate_device_id() {
        let device_id = CloudSyncService::generate_device_id().unwrap();
        assert!(device_id.contains("-local-"));
        assert!(device_id.len() > 10);
    }

    #[test]
    fn test_get_platform() {
        let platform = CloudSyncService::get_platform();
        assert!(!platform.is_empty());
        assert!(
            platform == "macos"
                || platform == "ios"
                || platform == "windows"
                || platform == "linux"
                || platform == "cli"
        );
    }
}
