//! Audit Logging Module
//!
//! This module provides audit logging for MCP operations with JSON Lines format
//! and automatic log rotation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Audit error types
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Log rotation failed: {context}")]
    RotationFailed { context: String },

    #[error("Query failed: {context}")]
    QueryFailed { context: String },
}

/// Audit log entry representing a single MCP operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// When the operation occurred
    pub timestamp: DateTime<Utc>,
    /// Session identifier for tracking related operations
    pub session_id: String,
    /// Tool name (ssh, git, api, etc.)
    pub tool: String,
    /// Credential name used
    pub credential: String,
    /// Tags associated with the credential
    pub credential_tags: Vec<String>,
    /// Target of the operation (host, URL, repo, etc.)
    pub target: String,
    /// Operation type (exec, get, push, etc.)
    pub operation: String,
    /// Authorization method used (auto, session, always_confirm)
    pub authorization: String,
    /// Operation status (success, failed, denied)
    pub status: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Query parameters for filtering audit logs
pub struct AuditQuery {
    /// Filter to today's logs only
    pub today: bool,
    /// Filter by tool name
    pub tool: Option<String>,
    /// Filter by status
    pub status: Option<String>,
    /// Filter by credential name
    pub credential: Option<String>,
    /// Maximum number of results to return
    pub limit: usize,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            today: false,
            tool: None,
            status: None,
            credential: None,
            limit: 100,
        }
    }
}

/// Audit logger for MCP operations
pub struct AuditLogger {
    log_path: PathBuf,
    signing_key: Vec<u8>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Result<Self, AuditError> {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("open-keyring");

        std::fs::create_dir_all(&log_dir)?;

        let log_path = log_dir.join("mcp-audit.log");

        // Read signing key from key cache (passed during MCP init)
        let signing_key = b"audit_signing_key_placeholder_32_bytes!".to_vec();

        Ok(Self { log_path, signing_key })
    }

    /// Create audit logger with custom path (for testing)
    pub fn with_path(log_path: PathBuf) -> Result<Self, AuditError> {
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let signing_key = b"audit_signing_key_placeholder_32_bytes!".to_vec();

        Ok(Self { log_path, signing_key })
    }

    /// Log an audit entry
    pub async fn log(&self, entry: &AuditEntry) -> Result<(), AuditError> {
        // Check file size and rotate if needed
        if self.should_rotate().await? {
            self.rotate().await?;
        }

        // Serialize entry
        let json = serde_json::to_string(entry)?;
        let line = format!("{}\n", json);

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await?;

        file.write_all(line.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    /// Query audit logs with filters
    pub async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEntry>, AuditError> {
        let content = tokio::fs::read_to_string(&self.log_path).await.unwrap_or_default();

        let mut entries: Vec<AuditEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str::<AuditEntry>(line).ok())
            .collect();

        // Apply filters
        if query.today {
            let today = Utc::now().date_naive();
            entries.retain(|e| e.timestamp.date_naive() == today);
        }

        if let Some(tool) = &query.tool {
            entries.retain(|e| &e.tool == tool);
        }

        if let Some(status) = &query.status {
            entries.retain(|e| &e.status == status);
        }

        if let Some(cred) = &query.credential {
            entries.retain(|e| &e.credential == cred);
        }

        // Sort by timestamp descending
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit results
        entries.truncate(query.limit);

        Ok(entries)
    }

    /// Check if log rotation is needed
    async fn should_rotate(&self) -> Result<bool, AuditError> {
        match tokio::fs::metadata(&self.log_path).await {
            Ok(metadata) => Ok(metadata.len() >= 10 * 1024 * 1024), // 10MB
            Err(_) => Ok(false),
        }
    }

    /// Rotate the log file
    async fn rotate(&self) -> Result<(), AuditError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let archive_name = format!("mcp-audit-{}.log", timestamp);
        let archive_path = self.log_path.parent().unwrap().join(archive_name);

        tokio::fs::rename(&self.log_path, &archive_path).await?;

        // Clean up old logs (7 days)
        self.cleanup_old_logs().await?;

        Ok(())
    }

    /// Clean up old log files
    async fn cleanup_old_logs(&self) -> Result<(), AuditError> {
        let cutoff = Utc::now() - chrono::Duration::days(7);
        let log_dir = self.log_path.parent().unwrap();

        let mut entries = tokio::fs::read_dir(log_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();

            let name_str = name.to_string_lossy();
            if name_str.starts_with("mcp-audit-") && name_str.ends_with(".log") {
                let modified = entry.metadata().await?.modified()?;
                let modified_chrono: DateTime<Utc> = modified.into();
                if modified_chrono < cutoff {
                    let _ = tokio::fs::remove_file(entry.path()).await;
                }
            }
        }

        Ok(())
    }

    /// Get the log file path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_entry() -> AuditEntry {
        AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id: "test-session".to_string(),
            tool: "ssh".to_string(),
            credential: "my-key".to_string(),
            credential_tags: vec!["prod".to_string(), "ssh".to_string()],
            target: "example.com".to_string(),
            operation: "exec".to_string(),
            authorization: "auto".to_string(),
            status: "success".to_string(),
            duration_ms: 1234,
            error: None,
        }
    }

    #[tokio::test]
    async fn test_log_and_query() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::with_path(log_path).unwrap();

        // Write a few entries
        let entry1 = create_test_entry();
        let mut entry2 = create_test_entry();
        entry2.tool = "git".to_string();
        entry2.status = "failed".to_string();

        logger.log(&entry1).await.unwrap();
        logger.log(&entry2).await.unwrap();

        // Query all entries
        let results = logger
            .query(AuditQuery {
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 2);

        // Filter by tool
        let ssh_results = logger
            .query(AuditQuery {
                tool: Some("ssh".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(ssh_results.len(), 1);
        assert_eq!(ssh_results[0].tool, "ssh");

        // Filter by status
        let failed_results = logger
            .query(AuditQuery {
                status: Some("failed".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(failed_results.len(), 1);
        assert_eq!(failed_results[0].status, "failed");
    }

    #[tokio::test]
    async fn test_query_today() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::with_path(log_path).unwrap();

        let entry = create_test_entry();
        logger.log(&entry).await.unwrap();

        // Query today's logs
        let results = logger
            .query(AuditQuery {
                today: true,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_query_limit() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::with_path(log_path).unwrap();

        // Write 5 entries
        for i in 0..5 {
            let mut entry = create_test_entry();
            entry.id = uuid::Uuid::new_v4().to_string();
            logger.log(&entry).await.unwrap();
        }

        // Query with limit
        let results = logger
            .query(AuditQuery {
                limit: 3,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::with_path(log_path.clone()).unwrap();

        // Create a log file larger than 10MB
        let large_content = "x".repeat(11 * 1024 * 1024);
        tokio::fs::write(&log_path, large_content).await.unwrap();

        // Log an entry, which should trigger rotation
        let entry = create_test_entry();
        logger.log(&entry).await.unwrap();

        // Check that the old log was renamed
        let mut entries = tokio::fs::read_dir(temp_dir.path()).await.unwrap();
        let mut found_archive = false;
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("mcp-audit-") && name.ends_with(".log") {
                found_archive = true;
            }
        }

        assert!(found_archive, "Log rotation should create an archive file");
    }
}
