//! MCP Server using rmcp crate
//!
//! This module implements the MCP (Model Context Protocol) server using the rmcp crate.
//! The server handles JSON-RPC communication via stdio transport.

use crate::error::Error;
use crate::mcp::audit::AuditLogger;
use crate::mcp::auth::{SessionCache, UsedTokenCache};
use crate::mcp::config::McpConfig;
// use crate::mcp::handlers::handle_ssh_exec; // TODO: Re-enable when handler is ready
use crate::mcp::tools::ssh::*;
use rmcp::{
    model::{ServerInfo, ServerCapabilities},
    ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for the database - using a placeholder until proper integration
/// In a real implementation, this would be the Vault or Database type
pub type Database = Arc<RwLock<()>>;

/// Type alias for key cache - using a placeholder until proper integration
pub type McpKeyCache = Arc<RwLock<()>>;

/// MCP Server errors
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Failed to start server: {0}")]
    ServerStart(String),

    #[error("Failed to build server: {0}")]
    ServerBuild(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
}

impl From<McpError> for Error {
    fn from(err: McpError) -> Self {
        Error::Mcp {
            context: err.to_string(),
        }
    }
}

/// MCP Server with rmcp
///
/// This server implements the Model Context Protocol using the rmcp crate,
/// providing stdio transport for communication with AI assistants.
///
/// # Example
///
/// ```no_run
/// use keyring_cli::mcp::server::McpServer;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let server = McpServer::new(
///         Arc::new(Default::default()),
///         Arc::new(Default::default()),
///         Default::default(),
///     );
///
///     server.run_stdio().await?;
///     Ok(())
/// }
/// ```
pub struct McpServer {
    /// Database instance for accessing stored credentials
    db: Arc<Database>,

    /// Key cache for caching decrypted keys
    key_cache: Arc<McpKeyCache>,

    /// MCP configuration
    config: McpConfig,

    /// Session cache for authorization
    session_cache: Arc<SessionCache>,

    /// Used tokens cache for replay protection
    used_tokens: Arc<UsedTokenCache>,

    /// Unique session ID for this server instance
    session_id: String,

    /// Audit logger
    audit_logger: AuditLogger,
}

impl McpServer {
    /// Create a new MCP server instance
    ///
    /// # Arguments
    ///
    /// * `db` - Database instance for credential access
    /// * `key_cache` - Key cache for caching decrypted keys
    /// * `config` - MCP configuration
    ///
    /// # Returns
    ///
    /// A new McpServer instance with a unique session ID
    pub fn new(
        db: Arc<Database>,
        key_cache: Arc<McpKeyCache>,
        config: McpConfig,
    ) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();

        let session_cache = Arc::new(SessionCache::new(
            config.session_cache.max_entries,
            config.session_cache.ttl_seconds,
        ));

        Self {
            db,
            key_cache,
            config,
            session_cache,
            used_tokens: Arc::new(UsedTokenCache::new()),
            session_id,
            audit_logger: AuditLogger::new(),
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Run the MCP server with stdio transport
    ///
    /// This method starts the server and communicates via stdin/stdout,
    /// which is the standard transport for MCP servers.
    ///
    /// # Returns
    ///
    /// Ok(()) if the server runs successfully, Err otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or encounters
    /// a communication error
    pub async fn run_stdio(self) -> std::result::Result<(), McpError> {
        use tokio::io::{stdin, stdout};

        // Create the server handler
        let handler = OpenKeyringHandler::from_server(self);

        // Serve with stdio transport
        let service = handler
            .serve((stdin(), stdout()))
            .await
            .map_err(|e| McpError::ServerStart(e.to_string()))?;

        // Wait for the server to finish
        service
            .waiting()
            .await
            .map_err(|e| McpError::ServerStart(e.to_string()))?;

        Ok(())
    }
}

/// The actual MCP server handler that implements rmcp::ServerHandler
///
/// This struct contains all the state and implements the tool methods.
#[derive(Clone)]
pub struct OpenKeyringHandler {
    db: Arc<Database>,
    key_cache: Arc<McpKeyCache>,
    config: McpConfig,
    session_cache: Arc<SessionCache>,
    used_tokens: Arc<UsedTokenCache>,
    session_id: String,
    audit_logger: Arc<AuditLogger>,
}

impl OpenKeyringHandler {
    /// Create a new handler from a server instance
    fn from_server(server: McpServer) -> Self {
        Self {
            db: server.db,
            key_cache: server.key_cache,
            config: server.config,
            session_cache: server.session_cache,
            used_tokens: server.used_tokens,
            session_id: server.session_id,
            audit_logger: Arc::new(server.audit_logger),
        }
    }

    /// Execute SSH command on remote host
    async fn ssh_exec_impl(&self, input: SshExecInput) -> String {
        // Log the tool execution
        let _ = self.audit_logger.log_event(
            "ssh_exec_called",
            &format!("credential={}, command={}", input.credential_name, input.command),
        );

        // Call the SSH handler
        // TODO: Implement proper SSH execution - this is a placeholder
        let output = SshExecOutput {
            stdout: "Not implemented yet".to_string(),
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 0,
        };
        match serde_json::to_string(&output) {
            Ok(output) => {
                serde_json::to_string(&output).unwrap_or_else(|_| r#"{"error":"Failed to serialize output"}"#.to_string())
            }
            Err(e) => {
                let error_msg = format!("SSH execution failed: {}", e);
                let _ = self.audit_logger.log_event("ssh_exec_failed", &error_msg);
                format!(r#"{{"error":"{}"}}"#, error_msg)
            }
        }
    }

    /// List SSH hosts
    async fn ssh_list_hosts_impl(&self, _input: SshListHostsInput) -> String {
        // Log the tool execution
        let _ = self.audit_logger.log_event("ssh_list_hosts_called", "");

        // This is a low-risk operation, so it doesn't require authorization
        let hosts: Vec<SshHostInfo> = vec![]; // TODO: Implement actual host listing

        let output = SshListHostsOutput { hosts };
        serde_json::to_string(&output).unwrap_or_else(|_| r#"{"error":"Failed to serialize output"}"#.to_string())
    }

    /// Check SSH connection
    async fn ssh_check_connection_impl(&self, input: SshCheckConnectionInput) -> String {
        // Log the tool execution
        let _ = self.audit_logger.log_event(
            "ssh_check_connection_called",
            &format!("credential={}", input.credential_name),
        );

        // This is a low-risk operation, so it doesn't require authorization
        let output = SshCheckConnectionOutput {
            connected: false,
            latency_ms: 0,
            error: Some("Not implemented yet".to_string()),
        };

        serde_json::to_string(&output).unwrap_or_else(|_| r#"{"error":"Failed to serialize output"}"#.to_string())
    }
}

/// Implement ServerHandler for the OpenKeyring MCP server
///
/// This trait is required by rmcp to define server capabilities and handle requests.
impl ServerHandler for OpenKeyringHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let db = Arc::new(RwLock::new(()));
        let key_cache = Arc::new(RwLock::new(()));
        let config = McpConfig::default();

        let server = McpServer::new(db, key_cache, config);

        assert!(!server.session_id().is_empty());
    }
}
