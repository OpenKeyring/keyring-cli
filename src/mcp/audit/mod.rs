use crate::error::KeyringError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: String,
    pub client_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub success: bool,
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug)]
pub struct SimpleAuditLogger {
    log_file_path: String,
    enabled: bool,
}

impl Default for SimpleAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleAuditLogger {
    pub fn new() -> Self {
        Self {
            log_file_path: std::env::var("OK_MCP_AUDIT_LOG")
                .unwrap_or_else(|_| "mcp_audit.log".to_string()),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn log_event(&self, event_type: &str, details: &str) -> Result<(), KeyringError> {
        if !self.enabled {
            return Ok(());
        }

        let event = AuditEvent {
            id: Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            client_id: None,
            timestamp: Utc::now(),
            details: serde_json::from_str(details)
                .unwrap_or_else(|_| serde_json::Value::String(details.to_string())),
            success: true,
            execution_time_ms: None,
        };

        self.write_audit_event(&event)?;
        Ok(())
    }

    pub fn log_tool_execution(
        &self,
        tool_name: &str,
        client_id: &str,
        arguments: &serde_json::Value,
        execution_time: Option<std::time::Duration>,
        success: bool,
    ) -> Result<(), KeyringError> {
        let event = AuditEvent {
            id: Uuid::new_v4().to_string(),
            event_type: "tool_execution".to_string(),
            client_id: Some(client_id.to_string()),
            timestamp: Utc::now(),
            details: serde_json::json!({
                "tool_name": tool_name,
                "arguments": arguments,
                "execution_time_ms": execution_time.map(|t| t.as_millis() as u64)
            }),
            success,
            execution_time_ms: execution_time.map(|t| t.as_millis() as u64),
        };

        self.write_audit_event(&event)?;
        Ok(())
    }

    pub fn log_authentication_event(
        &self,
        client_id: &str,
        event_type: &str,
        success: bool,
        additional_info: Option<serde_json::Value>,
    ) -> Result<(), KeyringError> {
        let event = AuditEvent {
            id: Uuid::new_v4().to_string(),
            event_type: format!("auth_{}", event_type),
            client_id: Some(client_id.to_string()),
            timestamp: Utc::now(),
            details: additional_info.unwrap_or(serde_json::json!({})),
            success,
            execution_time_ms: None,
        };

        self.write_audit_event(&event)?;
        Ok(())
    }

    fn write_audit_event(&self, event: &AuditEvent) -> Result<(), KeyringError> {
        let log_entry = format!(
            "[{}] {} | {} | success={} | client={} | details={}\n",
            event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            event.event_type,
            event.id,
            event.success,
            event.client_id.as_deref().unwrap_or("N/A"),
            serde_json::to_string(&event.details)?
        );

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        file.write_all(log_entry.as_bytes())
            .map_err(|e| KeyringError::IoError(e.to_string()))?;
        file.flush()
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn get_audit_logs(
        &self,
        _since: Option<DateTime<Utc>>,
    ) -> Result<Vec<AuditEvent>, KeyringError> {
        if !Path::new(&self.log_file_path).exists() {
            return Ok(Vec::new());
        }

        let _content = fs::read_to_string(&self.log_file_path)
            .map_err(|e| KeyringError::IoError(e.to_string()))?;

        // Parse audit logs (in a real implementation, this would parse structured logs)
        Ok(Vec::new()) // Placeholder for parsing implementation
    }

    pub fn clear_logs(&self) -> Result<(), KeyringError> {
        if Path::new(&self.log_file_path).exists() {
            fs::remove_file(&self.log_file_path)
                .map_err(|e| KeyringError::IoError(e.to_string()))?;
        }
        Ok(())
    }
}

// Re-export the async audit logger types for tests and external use
pub mod audit;

// Re-export the async logger types for tests
pub use audit::{AuditEntry as AsyncAuditEntry, AuditLogger as AsyncAuditLogger, AuditQuery};

// Export the simple logger as the default for internal use
pub use SimpleAuditLogger as AuditLogger;
