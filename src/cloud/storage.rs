//! Cloud Storage Operations
//!
//! Provides high-level storage operations for cloud synchronization using OpenDAL.

use anyhow::Result;
use crate::cloud::config::CloudConfig;
use crate::cloud::metadata::CloudMetadata;
use crate::cloud::provider::create_operator;
use opendal::Operator;

/// Cloud storage client for synchronization operations
///
/// Wraps an OpenDAL operator and provides methods for metadata
/// and record management in cloud storage.
pub struct CloudStorage {
    /// OpenDAL operator for cloud storage operations
    operator: Operator,
    /// Path to the metadata file in cloud storage
    metadata_path: String,
}

impl CloudStorage {
    /// Create a new CloudStorage instance from configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Cloud provider configuration
    ///
    /// # Returns
    ///
    /// Returns a `CloudStorage` instance or an error if configuration is invalid
    pub fn new(config: &CloudConfig) -> Result<Self> {
        let operator = create_operator(config)?;
        Ok(Self {
            operator,
            metadata_path: ".metadata.json".to_string(),
        })
    }

    /// Upload metadata to cloud storage
    ///
    /// Serializes the metadata to JSON and writes it to the metadata file.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Cloud metadata to upload
    pub async fn upload_metadata(&self, metadata: &CloudMetadata) -> Result<()> {
        let json = serde_json::to_string_pretty(metadata)?;
        self.operator.write(&self.metadata_path, json.into_bytes()).await?;
        Ok(())
    }

    /// Download metadata from cloud storage
    ///
    /// Reads and deserializes the metadata file.
    ///
    /// # Returns
    ///
    /// Returns the deserialized `CloudMetadata` or an error if the file
    /// doesn't exist or is invalid
    pub async fn download_metadata(&self) -> Result<CloudMetadata> {
        let buffer = self.operator.read(&self.metadata_path).await?;
        let json = String::from_utf8(buffer.to_vec())?;
        let metadata: CloudMetadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Check if metadata file exists in cloud storage
    ///
    /// # Returns
    ///
    /// Returns `true` if the metadata file exists, `false` otherwise
    pub async fn metadata_exists(&self) -> Result<bool> {
        Ok(self.operator.exists(&self.metadata_path).await?)
    }

    /// Upload a record to cloud storage
    ///
    /// Records are stored as `{id}-{device_id}.json` files.
    ///
    /// # Arguments
    ///
    /// * `id` - Record ID
    /// * `device_id` - Device identifier
    /// * `data` - Record data as JSON value
    pub async fn upload_record(
        &self,
        id: &str,
        device_id: &str,
        data: &serde_json::Value,
    ) -> Result<()> {
        let filename = format!("{}-{}.json", id, device_id);
        let json = serde_json::to_string_pretty(data)?;
        self.operator.write(&filename, json.into_bytes()).await?;
        Ok(())
    }

    /// Download a record from cloud storage
    ///
    /// # Arguments
    ///
    /// * `id` - Record ID
    /// * `device_id` - Device identifier
    ///
    /// # Returns
    ///
    /// Returns the deserialized record data or an error if the file
    /// doesn't exist or is invalid
    pub async fn download_record(
        &self,
        id: &str,
        device_id: &str,
    ) -> Result<serde_json::Value> {
        let filename = format!("{}-{}.json", id, device_id);
        let buffer = self.operator.read(&filename).await?;
        let json = String::from_utf8(buffer.to_vec())?;
        let data: serde_json::Value = serde_json::from_str(&json)?;
        Ok(data)
    }

    /// List all record files in cloud storage
    ///
    /// Excludes the metadata file and non-JSON files.
    ///
    /// # Returns
    ///
    /// Returns a vector of filenames (not full paths)
    pub async fn list_records(&self) -> Result<Vec<String>> {
        let entries = self.operator.list("/").await?;
        let mut files = Vec::new();

        for entry in entries {
            let path = entry.path().to_string();
            if path.ends_with(".json") && path != self.metadata_path {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Delete a record from cloud storage
    ///
    /// # Arguments
    ///
    /// * `id` - Record ID
    /// * `device_id` - Device identifier
    pub async fn delete_record(&self, id: &str, device_id: &str) -> Result<()> {
        let filename = format!("{}-{}.json", id, device_id);
        self.operator.delete(&filename).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud::config::CloudProvider;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cloud_storage_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let storage = CloudStorage::new(&config);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_cloud_storage_metadata_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = CloudConfig {
            provider: CloudProvider::ICloud,
            icloud_path: Some(temp_dir.path().to_path_buf()),
            ..Default::default()
        };

        let storage = CloudStorage::new(&config).unwrap();
        assert_eq!(storage.metadata_path, ".metadata.json");
    }
}
