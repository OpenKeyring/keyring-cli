//! Sync Configuration File Tests
//!
//! Test suite for sync configuration file management with YAML serialization.

use keyring_cli::config::SyncConfigFile;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_save_load_sync_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    let config = SyncConfigFile {
        sync_enabled: true,
        provider: "icloud".to_string(),
        icloud_path: Some("~/iCloud/open-keyring".to_string()),
        debounce_delay: 5,
        ..Default::default()
    };

    config.save(&config_path).unwrap();

    let loaded = SyncConfigFile::load(&config_path).unwrap();
    assert_eq!(loaded.provider, "icloud");
    assert!(loaded.sync_enabled);
    assert_eq!(
        loaded.icloud_path,
        Some("~/iCloud/open-keyring".to_string())
    );
    assert_eq!(loaded.debounce_delay, 5);
}

#[test]
fn test_default_config() {
    let config = SyncConfigFile::default();

    assert!(!config.sync_enabled);
    assert_eq!(config.provider, "icloud");
    assert_eq!(config.icloud_path, None);
    assert_eq!(config.debounce_delay, 5);
    assert!(!config.auto_sync);
}

#[test]
fn test_save_full_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("full_config.yaml");

    let config = SyncConfigFile {
        sync_enabled: true,
        provider: "dropbox".to_string(),
        icloud_path: Some("~/iCloud/open-keyring".to_string()),
        debounce_delay: 10,
        auto_sync: true,
    };

    config.save(&config_path).unwrap();

    // Verify file was created
    assert!(config_path.exists());

    // Verify content is valid YAML
    let contents = fs::read_to_string(&config_path).unwrap();
    assert!(contents.contains("sync_enabled"));
    assert!(contents.contains("provider"));
    assert!(contents.contains("dropbox"));

    // Load and verify all fields
    let loaded = SyncConfigFile::load(&config_path).unwrap();
    assert!(loaded.sync_enabled);
    assert_eq!(loaded.provider, "dropbox");
    assert_eq!(
        loaded.icloud_path,
        Some("~/iCloud/open-keyring".to_string())
    );
    assert_eq!(loaded.debounce_delay, 10);
    assert!(loaded.auto_sync);
}

#[test]
fn test_load_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.yaml");

    let result = SyncConfigFile::load(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_save_invalid_path() {
    let temp_dir = TempDir::new().unwrap();
    // Create a path that includes a nonexistent directory
    let config_path = temp_dir.path().join("nonexistent_dir/config.yaml");

    let config = SyncConfigFile::default();
    let result = config.save(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_yaml_serialization_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("format_test.yaml");

    let config = SyncConfigFile {
        sync_enabled: true,
        provider: "icloud".to_string(),
        icloud_path: Some("~/iCloud/open-keyring".to_string()),
        debounce_delay: 5,
        auto_sync: false,
    };

    config.save(&config_path).unwrap();

    let contents = fs::read_to_string(&config_path).unwrap();

    // Verify YAML structure
    assert!(contents.contains("sync_enabled: true"));
    assert!(contents.contains("provider: icloud"));
    assert!(contents.contains("debounce_delay: 5"));
    assert!(contents.contains("auto_sync: false"));
}

#[test]
fn test_partial_config_update() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("partial.yaml");

    // Create initial config
    let config = SyncConfigFile {
        sync_enabled: false,
        provider: "icloud".to_string(),
        icloud_path: None,
        debounce_delay: 5,
        auto_sync: false,
    };

    config.save(&config_path).unwrap();

    // Load and update
    let mut loaded = SyncConfigFile::load(&config_path).unwrap();
    loaded.sync_enabled = true;
    loaded.auto_sync = true;
    loaded.save(&config_path).unwrap();

    // Verify updates
    let final_config = SyncConfigFile::load(&config_path).unwrap();
    assert!(final_config.sync_enabled);
    assert_eq!(final_config.provider, "icloud"); // unchanged
    assert!(final_config.auto_sync);
    assert_eq!(final_config.debounce_delay, 5); // unchanged
}

#[test]
fn test_multiple_providers() {
    let providers = vec!["icloud", "dropbox", "google_drive", "webdav", "sftp"];

    for provider in providers {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(format!("{}_config.yaml", provider));

        let config = SyncConfigFile {
            sync_enabled: true,
            provider: provider.to_string(),
            icloud_path: Some("~/path/to/sync".to_string()),
            debounce_delay: 5,
            auto_sync: true,
        };

        config.save(&config_path).unwrap();

        let loaded = SyncConfigFile::load(&config_path).unwrap();
        assert_eq!(loaded.provider, provider);
    }
}

#[test]
fn test_debounce_delay_values() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("debounce_test.yaml");

    let test_values = vec![0, 1, 5, 10, 30, 60, 300];

    for delay in test_values {
        let config = SyncConfigFile {
            sync_enabled: true,
            provider: "icloud".to_string(),
            icloud_path: None,
            debounce_delay: delay,
            auto_sync: false,
        };

        config.save(&config_path).unwrap();

        let loaded = SyncConfigFile::load(&config_path).unwrap();
        assert_eq!(loaded.debounce_delay, delay);
    }
}
