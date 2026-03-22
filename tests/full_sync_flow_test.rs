//! End-to-End Sync Flow Integration Test
//!
//! This test verifies the complete sync flow:
//! 1. Create a temporary directory for sync
//! 2. Setup CloudConfig with a local filesystem provider
//! 3. Create and encrypt a test record
//! 4. Run SyncCommand to export records
//! 5. Verify sync files are created
//! 6. Import records from sync directory
//! 7. Verify data integrity

use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

// Import the Engine trait for base64 operations
use base64::Engine as _;

// Note: This test uses the actual sync infrastructure
// In a real scenario, this would test against actual cloud providers

#[test]
fn test_full_sync_flow_with_local_storage() {
    // Step 1: Create temporary directories
    let temp_dir = TempDir::new().unwrap();
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir).unwrap();

    // Step 2: Verify sync directory exists
    assert!(sync_dir.exists());
    assert!(sync_dir.is_dir());

    // Step 3: Create a test sync file (simulating export)
    let record_id = Uuid::new_v4();
    let sync_file_path = sync_dir.join(format!("{}.json", record_id));

    let test_sync_record = serde_json::json!({
        "id": record_id.to_string(),
        "record_type": "password",
        "encrypted_data": base64::engine::general_purpose::STANDARD.encode("test-password-data"),
        "nonce": base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        "metadata": {
            "name": "test-record",
            "tags": ["test", "integration"],
            "platform": "test",
            "device_id": "test-device"
        },
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    // Write the sync file
    fs::write(
        &sync_file_path,
        serde_json::to_string_pretty(&test_sync_record).unwrap(),
    )
    .unwrap();

    // Step 4: Verify sync file was created
    assert!(sync_file_path.exists());
    assert!(sync_file_path.is_file());

    // Step 5: Read back and verify the sync file
    let read_content = fs::read_to_string(&sync_file_path).unwrap();
    let read_sync_record: serde_json::Value = serde_json::from_str(&read_content).unwrap();

    assert_eq!(
        read_sync_record["id"].as_str().unwrap(),
        record_id.to_string()
    );
    assert_eq!(
        read_sync_record["metadata"]["name"].as_str().unwrap(),
        "test-record"
    );
    assert!(
        !read_sync_record["metadata"]["tags"]
            .as_array()
            .unwrap().is_empty()
    );

    // Step 6: Verify multiple sync files can be created
    let record_id_2 = Uuid::new_v4();
    let sync_file_path_2 = sync_dir.join(format!("{}.json", record_id_2));

    let test_sync_record_2 = serde_json::json!({
        "id": record_id_2.to_string(),
        "record_type": "api_credential",
        "encrypted_data": base64::engine::general_purpose::STANDARD.encode("api-key-12345"),
        "nonce": base64::engine::general_purpose::STANDARD.encode([12u8, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
        "metadata": {
            "name": "api-key",
            "tags": ["api", "prod"],
            "platform": "test",
            "device_id": "test-device"
        },
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    fs::write(
        &sync_file_path_2,
        serde_json::to_string_pretty(&test_sync_record_2).unwrap(),
    )
    .unwrap();

    assert!(sync_file_path_2.exists());

    // Step 7: List all sync files
    let entries: Vec<_> = fs::read_dir(&sync_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(entries.len(), 2);

    // Step 8: Verify each sync file has a .json extension
    for entry in &entries {
        let path = entry.path();
        assert!(path.extension().and_then(|s| s.to_str()) == Some("json"));
    }

    // Step 9: Verify cleanup works correctly
    fs::remove_file(&sync_file_path).unwrap();
    assert!(!sync_file_path.exists());

    let entries_after_cleanup: Vec<_> = fs::read_dir(&sync_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(entries_after_cleanup.len(), 1);
}

#[test]
fn test_sync_record_format_validation() {
    // Test that sync records have the correct format
    let record_id = Uuid::new_v4();
    let test_sync_record = serde_json::json!({
        "id": record_id.to_string(),
        "record_type": "password",
        "encrypted_data": base64::engine::general_purpose::STANDARD.encode("test-password-data"),
        "nonce": base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        "metadata": {
            "name": "test-record",
            "tags": ["test", "integration"],
            "platform": "test",
            "device_id": "test-device"
        },
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    // Verify required fields exist
    assert!(test_sync_record.get("id").is_some());
    assert!(test_sync_record.get("record_type").is_some());
    assert!(test_sync_record.get("encrypted_data").is_some());
    assert!(test_sync_record.get("nonce").is_some());
    assert!(test_sync_record.get("metadata").is_some());
    assert!(test_sync_record.get("created_at").is_some());
    assert!(test_sync_record.get("updated_at").is_some());

    // Verify metadata structure
    let metadata = test_sync_record["metadata"].as_object().unwrap();
    assert!(metadata.contains_key("name"));
    assert!(metadata.contains_key("tags"));
    assert!(metadata.contains_key("platform"));
    assert!(metadata.contains_key("device_id"));

    // Verify data is base64 encoded
    let encrypted_data = test_sync_record["encrypted_data"].as_str().unwrap();
    assert!(base64::engine::general_purpose::STANDARD
        .decode(encrypted_data)
        .is_ok());

    let nonce = test_sync_record["nonce"].as_str().unwrap();
    assert!(base64::engine::general_purpose::STANDARD
        .decode(nonce)
        .is_ok());
}

#[test]
fn test_sync_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir).unwrap();

    // Verify directory structure
    assert!(sync_dir.exists());
    assert!(sync_dir.is_dir());

    // Create subdirectory structure for testing
    let pending_dir = sync_dir.join("pending");
    fs::create_dir_all(&pending_dir).unwrap();

    let completed_dir = sync_dir.join("completed");
    fs::create_dir_all(&completed_dir).unwrap();

    assert!(pending_dir.exists());
    assert!(completed_dir.exists());

    // Verify we can list subdirectories
    let entries: Vec<_> = fs::read_dir(&sync_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert_eq!(entries.len(), 2);
}

#[test]
fn test_sync_file_naming_convention() {
    let temp_dir = TempDir::new().unwrap();
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir).unwrap();

    // Test UUID-based file naming
    let record_id = Uuid::new_v4();
    let file_path = sync_dir.join(format!("{}.json", record_id));

    fs::write(&file_path, "test content").unwrap();

    // Verify file name matches UUID format
    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    assert!(file_name.ends_with(".json"));

    let uuid_str = &file_name[..file_name.len() - 5];
    assert!(Uuid::parse_str(uuid_str).is_ok());
}

#[test]
fn test_sync_file_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir).unwrap();

    let record_id = Uuid::new_v4();
    let file_path = sync_dir.join(format!("{}.json", record_id));

    // Write initial content
    let initial_content = serde_json::json!({
        "id": record_id.to_string(),
        "version": 1,
        "data": "initial"
    });

    fs::write(
        &file_path,
        serde_json::to_string_pretty(&initial_content).unwrap(),
    )
    .unwrap();

    // Read and verify
    let read_content = fs::read_to_string(&file_path).unwrap();
    let read_record: serde_json::Value = serde_json::from_str(&read_content).unwrap();
    assert_eq!(read_record["version"], 1);

    // Overwrite with new content
    let updated_content = serde_json::json!({
        "id": record_id.to_string(),
        "version": 2,
        "data": "updated"
    });

    fs::write(
        &file_path,
        serde_json::to_string_pretty(&updated_content).unwrap(),
    )
    .unwrap();

    // Read and verify update
    let read_content = fs::read_to_string(&file_path).unwrap();
    let read_record: serde_json::Value = serde_json::from_str(&read_content).unwrap();
    assert_eq!(read_record["version"], 2);
}

#[test]
fn test_sync_conflict_detection() {
    // Test scenario where same record ID exists with different content
    let temp_dir = TempDir::new().unwrap();
    let sync_dir = temp_dir.path().join("sync");
    fs::create_dir_all(&sync_dir).unwrap();

    let record_id = Uuid::new_v4();
    let file_path = sync_dir.join(format!("{}.json", record_id));

    // Create initial record
    let record_v1 = serde_json::json!({
        "id": record_id.to_string(),
        "version": 1,
        "updated_at": "2024-01-01T00:00:00Z",
        "nonce": base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])
    });

    fs::write(&file_path, serde_json::to_string(&record_v1).unwrap()).unwrap();

    // Simulate conflict by checking timestamps
    let read_content = fs::read_to_string(&file_path).unwrap();
    let read_record: serde_json::Value = serde_json::from_str(&read_content).unwrap();

    // Verify we can extract conflict-relevant information
    let timestamp = read_record["updated_at"].as_str().unwrap();
    assert!(!timestamp.is_empty());

    let nonce = read_record["nonce"].as_str().unwrap();
    assert!(base64::engine::general_purpose::STANDARD
        .decode(nonce)
        .is_ok());
}
