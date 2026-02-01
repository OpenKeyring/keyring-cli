//! Integration tests for Audit Logging module

use keyring_cli::mcp::audit::{AsyncAuditEntry as AuditEntry, AsyncAuditLogger as AuditLogger, AuditQuery};
use tempfile::TempDir;

fn create_test_entry(tool: &str, status: &str) -> AuditEntry {
    AuditEntry {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        session_id: "test-session-123".to_string(),
        tool: tool.to_string(),
        credential: "test-credential".to_string(),
        credential_tags: vec!["test".to_string(), "integration".to_string()],
        target: "test-target.example.com".to_string(),
        operation: "test_operation".to_string(),
        authorization: "session".to_string(),
        status: status.to_string(),
        duration_ms: 100,
        error: None,
    }
}

#[tokio::test]
async fn test_audit_log_write_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path.clone()).unwrap();

    // Create and log an entry
    let entry = create_test_entry("ssh", "success");
    logger.log(&entry).await.expect("Failed to log entry");

    // Verify log file exists
    assert!(log_path.exists(), "Log file should be created");

    // Read and parse the log file
    let content = tokio::fs::read_to_string(&log_path)
        .await
        .expect("Failed to read log file");

    let parsed_entry: AuditEntry =
        serde_json::from_str(&content.trim()).expect("Failed to parse entry");

    assert_eq!(parsed_entry.id, entry.id);
    assert_eq!(parsed_entry.tool, "ssh");
    assert_eq!(parsed_entry.status, "success");
}

#[tokio::test]
async fn test_audit_query_all() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log multiple entries
    logger.log(&create_test_entry("ssh", "success")).await.unwrap();
    logger.log(&create_test_entry("git", "success")).await.unwrap();
    logger.log(&create_test_entry("api", "failed")).await.unwrap();

    // Query all entries
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 3, "Should return all 3 entries");

    // Verify order (most recent first)
    assert!(results[0].timestamp > results[1].timestamp);
    assert!(results[1].timestamp > results[2].timestamp);
}

#[tokio::test]
async fn test_audit_query_by_tool() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log entries with different tools
    logger.log(&create_test_entry("ssh", "success")).await.unwrap();
    logger.log(&create_test_entry("git", "success")).await.unwrap();
    logger.log(&create_test_entry("ssh", "failed")).await.unwrap();

    // Query SSH entries only
    let results = logger
        .query(AuditQuery {
            tool: Some("ssh".to_string()),
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 2, "Should return 2 SSH entries");
    assert!(results.iter().all(|e| e.tool == "ssh"));
}

#[tokio::test]
async fn test_audit_query_by_status() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log entries with different statuses
    logger.log(&create_test_entry("ssh", "success")).await.unwrap();
    logger.log(&create_test_entry("git", "failed")).await.unwrap();
    logger.log(&create_test_entry("api", "failed")).await.unwrap();

    // Query failed entries only
    let results = logger
        .query(AuditQuery {
            status: Some("failed".to_string()),
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 2, "Should return 2 failed entries");
    assert!(results.iter().all(|e| e.status == "failed"));
}

#[tokio::test]
async fn test_audit_query_by_credential() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log entries with different credentials
    let mut entry1 = create_test_entry("ssh", "success");
    entry1.credential = "prod-key".to_string();
    logger.log(&entry1).await.unwrap();

    let mut entry2 = create_test_entry("ssh", "success");
    entry2.credential = "dev-key".to_string();
    logger.log(&entry2).await.unwrap();

    // Query by credential
    let results = logger
        .query(AuditQuery {
            credential: Some("prod-key".to_string()),
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 1, "Should return 1 entry for prod-key");
    assert_eq!(results[0].credential, "prod-key");
}

#[tokio::test]
async fn test_audit_query_today() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log an entry for today
    let entry = create_test_entry("ssh", "success");
    logger.log(&entry).await.unwrap();

    // Query today's entries
    let results = logger
        .query(AuditQuery {
            today: true,
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 1, "Should return today's entry");
}

#[tokio::test]
async fn test_audit_query_limit() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log 10 entries
    for _ in 0..10 {
        logger.log(&create_test_entry("ssh", "success")).await.unwrap();
    }

    // Query with limit of 5
    let results = logger
        .query(AuditQuery {
            limit: 5,
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 5, "Should return only 5 entries");
}

#[tokio::test]
async fn test_audit_query_combined_filters() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log various entries
    logger.log(&create_test_entry("ssh", "success")).await.unwrap();
    logger.log(&create_test_entry("ssh", "failed")).await.unwrap();
    logger.log(&create_test_entry("git", "success")).await.unwrap();
    logger.log(&create_test_entry("git", "failed")).await.unwrap();

    // Query: ssh AND success
    let results = logger
        .query(AuditQuery {
            tool: Some("ssh".to_string()),
            status: Some("success".to_string()),
            ..Default::default()
        })
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 1, "Should return 1 ssh+success entry");
    assert_eq!(results[0].tool, "ssh");
    assert_eq!(results[0].status, "success");
}

#[tokio::test]
async fn test_audit_log_rotation() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path.clone()).unwrap();

    // Create a log file larger than 10MB to trigger rotation
    let large_content = "x".repeat(11 * 1024 * 1024); // 11MB
    tokio::fs::write(&log_path, large_content)
        .await
        .expect("Failed to write large content");

    // Log an entry, which should trigger rotation
    let entry = create_test_entry("ssh", "success");
    logger.log(&entry).await.expect("Failed to log entry");

    // Check that the old log was renamed to archive
    let mut entries = tokio::fs::read_dir(temp_dir.path())
        .await
        .expect("Failed to read directory");
    let mut found_archive = false;
    let mut found_current_log = false;

    while let Some(entry) = entries
        .next_entry()
        .await
        .expect("Failed to read directory entry")
    {
        let name = entry.file_name().to_string_lossy().to_string();
        // Archive files are named with hardcoded "mcp-audit-" prefix
        if name.starts_with("mcp-audit-") && name.ends_with(".log") {
            found_archive = true;
        }
        if name == "test-audit.log" {
            found_current_log = true;
        }
    }

    assert!(
        found_archive,
        "Old log should be renamed to archive format"
    );
    assert!(found_current_log, "New log file should be created");
}

#[tokio::test]
async fn test_audit_log_entry_with_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Create an entry with an error
    let mut entry = create_test_entry("ssh", "failed");
    entry.error = Some("Connection refused".to_string());

    logger.log(&entry).await.expect("Failed to log entry");

    // Read back the entry
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, "failed");
    assert_eq!(results[0].error, Some("Connection refused".to_string()));
}

#[tokio::test]
async fn test_audit_log_multiple_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log entries from different sessions
    let mut entry1 = create_test_entry("ssh", "success");
    entry1.session_id = "session-1".to_string();

    let mut entry2 = create_test_entry("ssh", "success");
    entry2.session_id = "session-2".to_string();

    logger.log(&entry1).await.unwrap();
    logger.log(&entry2).await.unwrap();

    // Query all entries
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 2);
    let session_ids: Vec<&str> = results.iter().map(|e| e.session_id.as_str()).collect();
    assert!(session_ids.contains(&"session-1"));
    assert!(session_ids.contains(&"session-2"));
}

#[tokio::test]
async fn test_audit_log_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Query on empty log file should return empty results
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 0, "Empty log should return no entries");
}

#[tokio::test]
async fn test_audit_log_duration_tracking() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Create entries with different durations
    let mut entry1 = create_test_entry("ssh", "success");
    entry1.duration_ms = 100;

    let mut entry2 = create_test_entry("ssh", "success");
    entry2.duration_ms = 5000;

    logger.log(&entry1).await.unwrap();
    logger.log(&entry2).await.unwrap();

    // Query and verify durations
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 2);
    let durations: Vec<u64> = results.iter().map(|e| e.duration_ms).collect();
    assert!(durations.contains(&100));
    assert!(durations.contains(&5000));
}

#[tokio::test]
async fn test_audit_authorization_methods() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("test-audit.log");
    let logger = AuditLogger::with_path(log_path).unwrap();

    // Log entries with different authorization methods
    let mut entry1 = create_test_entry("ssh", "success");
    entry1.authorization = "auto".to_string();

    let mut entry2 = create_test_entry("ssh", "success");
    entry2.authorization = "session".to_string();

    let mut entry3 = create_test_entry("ssh", "success");
    entry3.authorization = "always_confirm".to_string();

    logger.log(&entry1).await.unwrap();
    logger.log(&entry2).await.unwrap();
    logger.log(&entry3).await.unwrap();

    // Query all entries
    let results = logger
        .query(AuditQuery::default())
        .await
        .expect("Failed to query entries");

    assert_eq!(results.len(), 3);
    let auth_methods: Vec<&str> = results
        .iter()
        .map(|e| e.authorization.as_str())
        .collect();
    assert!(auth_methods.contains(&"auto"));
    assert!(auth_methods.contains(&"session"));
    assert!(auth_methods.contains(&"always_confirm"));
}
