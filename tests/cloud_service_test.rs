// tests/cloud_service_test.rs
use keyring_cli::sync::{cloud_service::{CloudSyncService, SyncDirection}};
use keyring_cli::cloud::{config::CloudConfig, CloudProvider};
use tempfile::TempDir;

#[tokio::test]
async fn test_initialize_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let config = CloudConfig {
        provider: CloudProvider::ICloud,
        icloud_path: Some(temp_dir.path().to_path_buf()),
        ..Default::default()
    };

    let service = CloudSyncService::new(&config, &[1u8; 32]).unwrap();

    // First call should create metadata
    service.initialize_metadata().await.unwrap();
    assert!(service.storage.metadata_exists().await.unwrap());

    // Second call should skip creation
    service.initialize_metadata().await.unwrap();
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

    let _stats = service.sync(SyncDirection::Upload).await.unwrap();
    // Should not error
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

    let _stats = service.sync(SyncDirection::Download).await.unwrap();
    // Should not error
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

    let _stats = service.sync(SyncDirection::Both).await.unwrap();
    // Should not error
}
