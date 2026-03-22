//! MCP Configuration Tests
//!
//! Tests for MCP configuration module including loading, saving, and default values.

use keyring_cli::mcp::config::{McpConfig, SessionCacheConfig};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_values() {
    let config = McpConfig::default();

    // Check default limits
    assert_eq!(config.max_concurrent_requests, 10);
    assert_eq!(config.max_response_size_ssh, 10 * 1024 * 1024); // 10MB
    assert_eq!(config.max_response_size_api, 5 * 1024 * 1024); // 5MB

    // Check session cache defaults
    assert_eq!(config.session_cache.max_entries, 100);
    assert_eq!(config.session_cache.ttl_seconds, 3600);
}

#[test]
fn test_session_cache_config_default() {
    let cache_config = SessionCacheConfig::default();

    assert_eq!(cache_config.max_entries, 100);
    assert_eq!(cache_config.ttl_seconds, 3600);
}

#[test]
fn test_roundtrip_serialization() {
    let original = McpConfig {
        max_concurrent_requests: 20,
        max_response_size_ssh: 20 * 1024 * 1024,
        max_response_size_api: 10 * 1024 * 1024,
        session_cache: SessionCacheConfig {
            max_entries: 200,
            ttl_seconds: 7200,
        },
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: McpConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_load_or_default_creates_default() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    // Load should create default when file doesn't exist
    let config = McpConfig::load_or_default(&config_path).unwrap();

    assert_eq!(config.max_concurrent_requests, 10);
    assert_eq!(config.max_response_size_ssh, 10 * 1024 * 1024);
    assert_eq!(config.max_response_size_api, 5 * 1024 * 1024);
    assert_eq!(config.session_cache.max_entries, 100);
    assert_eq!(config.session_cache.ttl_seconds, 3600);

    // Verify file was created
    assert!(config_path.exists());
}

#[test]
fn test_load_existing_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    // Create a custom config
    let custom_config = McpConfig {
        max_concurrent_requests: 15,
        max_response_size_ssh: 15 * 1024 * 1024,
        max_response_size_api: 8 * 1024 * 1024,
        session_cache: SessionCacheConfig {
            max_entries: 150,
            ttl_seconds: 1800,
        },
    };

    // Save it
    custom_config.save(&config_path).unwrap();

    // Load it back
    let loaded_config = McpConfig::load_or_default(&config_path).unwrap();

    assert_eq!(loaded_config.max_concurrent_requests, 15);
    assert_eq!(loaded_config.max_response_size_ssh, 15 * 1024 * 1024);
    assert_eq!(loaded_config.max_response_size_api, 8 * 1024 * 1024);
    assert_eq!(loaded_config.session_cache.max_entries, 150);
    assert_eq!(loaded_config.session_cache.ttl_seconds, 1800);
}

#[test]
fn test_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    let config = McpConfig {
        max_concurrent_requests: 25,
        max_response_size_ssh: 12 * 1024 * 1024,
        max_response_size_api: 6 * 1024 * 1024,
        session_cache: SessionCacheConfig {
            max_entries: 120,
            ttl_seconds: 5400,
        },
    };

    // Save the config
    config.save(&config_path).unwrap();

    // Verify file exists
    assert!(config_path.exists());

    // Load it back
    let loaded_config = McpConfig::load(&config_path).unwrap();

    assert_eq!(config, loaded_config);
}

#[test]
fn test_invalid_json_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    // Write invalid JSON
    fs::write(&config_path, "{ invalid json }").unwrap();

    // Should return error, not panic
    let result = McpConfig::load(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_load_or_default_fallback_on_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    // Write invalid JSON
    fs::write(&config_path, "{ invalid json }").unwrap();

    // Should fall back to default
    let config = McpConfig::load_or_default(&config_path).unwrap();

    assert_eq!(config.max_concurrent_requests, 10);
}

#[test]
fn test_config_file_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("mcp-config.json");

    let config = McpConfig::default();
    config.save(&config_path).unwrap();

    // Read the file and check it's valid JSON
    let contents = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

    // Check structure
    assert!(parsed.is_object());
    assert!(parsed.get("max_concurrent_requests").is_some());
    assert!(parsed.get("max_response_size_ssh").is_some());
    assert!(parsed.get("max_response_size_api").is_some());
    assert!(parsed.get("session_cache").is_some());

    // Check session cache structure
    let session_cache = parsed.get("session_cache").unwrap();
    assert!(session_cache.is_object());
    assert!(session_cache.get("max_entries").is_some());
    assert!(session_cache.get("ttl_seconds").is_some());
}
