pub mod audit;
pub mod auth;
pub mod authorization;
pub mod config;
pub mod executors;
pub mod handlers;
pub mod key_cache;
pub mod lock;
pub mod server;
pub mod tools;

// Re-export public types
pub use audit::{AuditEvent, AuditLogger};
pub use auth::{AuthDecision, ConfirmationToken, EnvTag, OperationType, PolicyEngine, RiskTag, SessionCache, UsedTokenCache};
pub use authorization::{AuthManager, AuthToken};
pub use config::McpConfig;
pub use executors::ExecutionResult;
pub use handlers::{handle_ssh_exec, HandlerError};
pub use key_cache::{KeyCacheError, McpKeyCache};
pub use lock::{is_locked, McpLock};
pub use server::{McpServer, McpError};
pub use tools::{McpToolRegistry, ToolDefinition};

pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const MAX_TOOL_EXECUTION_TIME: u64 = 30; // seconds
