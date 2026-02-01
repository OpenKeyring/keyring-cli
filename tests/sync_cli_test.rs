//! CLI sync command tests
//!
//! TDD approach: Tests written first (RED), implementation follows (GREEN)

#![cfg(feature = "test-env")]

use clap::Parser;
use keyring_cli::cli::commands::sync::SyncCommand;
use serial_test::serial;
use tempfile::TempDir;

/// Helper to set up test environment and clean up afterwards
struct TestEnv {
    _temp_dir: TempDir,
}

impl TestEnv {
    fn setup(test_name: &str) -> Self {
        // Clean up any existing environment variables first
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join(format!("config_{}", test_name));
        let data_dir = temp_dir.path().join(format!("data_{}", test_name));
        std::env::set_var("OK_CONFIG_DIR", config_dir.to_str().unwrap());
        std::env::set_var("OK_DATA_DIR", data_dir.to_str().unwrap());
        std::env::set_var("OK_MASTER_PASSWORD", "test-password");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        Self {
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        // Clean up environment variables
        std::env::remove_var("OK_CONFIG_DIR");
        std::env::remove_var("OK_DATA_DIR");
        std::env::remove_var("OK_MASTER_PASSWORD");
    }
}

#[serial]
#[test]
fn test_sync_command_parsing() {
    let args = vec!["sync".to_string()];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.direction, "both");
    assert_eq!(command.dry_run, false);
    assert_eq!(command.status, false);
    assert_eq!(command.config, false);
}

#[serial]
#[test]
fn test_sync_command_with_direction() {
    let args = vec![
        "sync".to_string(),
        "--direction".to_string(),
        "up".to_string(),
    ];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.direction, "up");
    assert_eq!(command.dry_run, false);
}

#[serial]
#[test]
fn test_sync_command_with_dry_run() {
    let args = vec!["sync".to_string(), "--dry-run".to_string()];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.direction, "both");
    assert_eq!(command.dry_run, true);
}

#[serial]
#[test]
fn test_sync_status_command() {
    let args = vec!["sync".to_string(), "--status".to_string()];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.status, true);
}

#[serial]
#[test]
fn test_sync_config_command() {
    let args = vec![
        "sync".to_string(),
        "--config".to_string(),
        "--provider".to_string(),
        "dropbox".to_string(),
    ];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.config, true);
    assert_eq!(command.provider, Some("dropbox".to_string()));
}

#[serial]
#[test]
fn test_sync_config_without_provider() {
    let args = vec!["sync".to_string(), "--config".to_string()];
    let command = SyncCommand::try_parse_from(&args).unwrap();
    assert_eq!(command.config, true);
    assert_eq!(command.provider, None);
}

#[serial]
#[test]
fn test_sync_direction_validation() {
    // Test valid directions
    for direction in &["up", "down", "both"] {
        let args = vec![
            "sync".to_string(),
            "--direction".to_string(),
            direction.to_string(),
        ];
        let command = SyncCommand::try_parse_from(&args);
        assert!(command.is_ok(), "Direction '{}' should be valid", direction);
    }
}

#[serial]
#[test]
fn test_sync_execute_sync() {
    let _env = TestEnv::setup("execute_sync");

    let command = SyncCommand {
        status: false,
        config: false,
        provider: None,
        direction: "both".to_string(),
        dry_run: false,
    };

    let result = command.execute();
    assert!(result.is_ok(), "Sync execution should succeed");
}

#[serial]
#[test]
fn test_sync_execute_status() {
    let _env = TestEnv::setup("execute_status");

    let command = SyncCommand {
        status: true,
        config: false,
        provider: None,
        direction: "both".to_string(),
        dry_run: false,
    };

    let result = command.execute();
    assert!(result.is_ok(), "Status execution should succeed");
}

#[serial]
#[test]
fn test_sync_execute_config() {
    let _env = TestEnv::setup("execute_config");

    let command = SyncCommand {
        status: false,
        config: true,
        provider: Some("icloud".to_string()),
        direction: "both".to_string(),
        dry_run: false,
    };

    let result = command.execute();
    assert!(result.is_ok(), "Config execution should succeed");
}
