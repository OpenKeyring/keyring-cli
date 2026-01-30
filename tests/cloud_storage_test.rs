// tests/cloud_storage_test.rs
use keyring_cli::cloud::{CloudStorage, config::{CloudConfig, CloudProvider}};
use keyring_cli::cloud::metadata::CloudMetadata;
use tempfile::TempDir;
use base64::prelude::*;

#[tokio::test]
async fn test_upload_download_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let storage = CloudStorage::new(&config).unwrap();
    let metadata = CloudMetadata::default();

    storage.upload_metadata(&metadata).await.unwrap();
    assert!(storage.metadata_exists().await.unwrap());

    let downloaded = storage.download_metadata().await.unwrap();
    assert_eq!(downloaded.format_version, "1.0");
}

#[tokio::test]
async fn test_upload_download_record() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let storage = CloudStorage::new(&config).unwrap();
    let record = serde_json::json!({
        "id": "test-id",
        "version": 1,
        "encrypted_payload": BASE64_STANDARD.encode(b"test-data"),
    });

    storage.upload_record("test-id", "device-1", &record).await.unwrap();

    let files = storage.list_records().await.unwrap();
    assert!(files.iter().any(|f| f.contains("test-id")));

    let downloaded = storage.download_record("test-id", "device-1").await.unwrap();
    assert_eq!(downloaded["id"], "test-id");
}

#[tokio::test]
async fn test_delete_record() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let storage = CloudStorage::new(&config).unwrap();
    let record = serde_json::json!({"id": "test-id"});

    storage.upload_record("test-id", "device-1", &record).await.unwrap();
    storage.delete_record("test-id", "device-1").await.unwrap();

    let files = storage.list_records().await.unwrap();
    assert!(!files.iter().any(|f| f.contains("test-id")));
}

#[tokio::test]
async fn test_list_records_empty() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let storage = CloudStorage::new(&config).unwrap();
    let files = storage.list_records().await.unwrap();
    assert!(files.is_empty());
}

#[tokio::test]
async fn test_metadata_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let storage = CloudStorage::new(&config).unwrap();
    assert!(!storage.metadata_exists().await.unwrap());
}
