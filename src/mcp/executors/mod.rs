//! MCP Tool Executors
//!
//! This module contains executors for different types of MCP tools:
//! - API executor for HTTP requests
//! - SSH executor for remote command execution
//! - Git executor for version control operations (using gix pure Rust implementation)

pub mod api;
pub mod git;  // Git executor using gix (pure Rust)
pub mod ssh;  // SSH tool definitions (input/output structs)
pub mod ssh_executor;  // SSH executor implementation

use crate::error::KeyringError;
use crate::mcp::audit::AuditLogger;
use crate::mcp::tools::McpToolRegistry;
use serde_json::Value;
use std::time::Duration;

// Re-export API executor types
pub use api::{ApiError, ApiExecutor, ApiResponse};
pub use git::{GitCloneOutput, GitError, GitExecutor, GitPullOutput, GitPushOutput};
pub use ssh::*;  // Re-export SSH tool definitions
pub use ssh_executor::{SshError, SshExecOutput as SshExecutorOutput, SshExecutor};  // Re-export SSH executor

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub execution_time: Duration,
}

pub struct AsyncToolExecutor {
    registry: McpToolRegistry,
    #[allow(dead_code)]
    max_execution_time: Duration,
    audit_logger: AuditLogger,
}

impl AsyncToolExecutor {
    pub fn new(registry: McpToolRegistry) -> Self {
        Self {
            registry,
            max_execution_time: Duration::from_secs(30),
            audit_logger: AuditLogger::new(),
        }
    }

    pub async fn execute_tool(
        &mut self,
        tool_name: &str,
        args: Value,
        client_id: &str,
    ) -> Result<ExecutionResult, KeyringError> {
        let start_time = std::time::Instant::now();

        // Get tool definition
        let _tool =
            self.registry
                .get_tool(tool_name)
                .ok_or_else(|| KeyringError::ToolNotFound {
                    tool_name: tool_name.to_string(),
                })?;

        // Log tool execution
        self.audit_logger
            .log_tool_execution(tool_name, client_id, &args, None, true)?;

        // Execute the tool (mock implementation for now)
        let result = match tool_name {
            "generate_password" => self.execute_generate_password(args.clone()),
            "list_records" => self.execute_list_records(),
            _ => Err(KeyringError::ToolNotFound {
                tool_name: tool_name.to_string(),
            }),
        };

        let execution_time = start_time.elapsed();

        match &result {
            Ok(execution_result) => {
                self.audit_logger.log_tool_execution(
                    tool_name,
                    client_id,
                    &args,
                    Some(execution_time),
                    execution_result.success,
                )?;
            }
            Err(_) => {
                self.audit_logger.log_tool_execution(
                    tool_name,
                    client_id,
                    &args,
                    Some(execution_time),
                    false,
                )?;
            }
        }

        result.map(|mut r| {
            r.execution_time = execution_time;
            r
        })
    }

    fn execute_generate_password(&self, args: Value) -> Result<ExecutionResult, KeyringError> {
        let length = args["length"].as_u64().unwrap_or(16) as usize;
        let include_symbols = args["include_symbols"].as_bool().unwrap_or(true);

        // In a real implementation, this would generate a secure password
        let password = "generated_password".repeat(length / 20 + 1);

        Ok(ExecutionResult {
            success: true,
            output: serde_json::json!({
                "password": password[..length.min(password.len())],
                "length": length,
                "include_symbols": include_symbols
            }),
            error: None,
            execution_time: Duration::from_millis(10),
        })
    }

    fn execute_list_records(&self) -> Result<ExecutionResult, KeyringError> {
        // Mock data
        Ok(ExecutionResult {
            success: true,
            output: serde_json::json!([]),
            error: None,
            execution_time: Duration::from_millis(5),
        })
    }
}
